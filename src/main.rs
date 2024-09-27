#![no_std]
#![no_main]

mod button;
mod led;
mod sensors;

use button::ButtonDirection;
use embassy_executor::Spawner;
use embassy_nrf::{self as hal, gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull}, peripherals::TWISPI0, temp, twim::Twim};
use embassy_nrf::temp::Temp;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use embassy_time::Delay;
use futures::{select_biased, FutureExt};
use hal::twim;
use led::LedRow;
use lsm303agr::{interface::I2cInterface, mode::MagContinuous, Lsm303agr};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use sensors::DataStore;

static CHANNEL: Channel<ThreadModeRawMutex, ButtonDirection, 1> = Channel::new();
type SensorType<'a> = Lsm303agr<I2cInterface<Twim<'a, TWISPI0>>, MagContinuous>;

hal::bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<hal::peripherals::TWISPI0>;
    TEMP => temp::InterruptHandler;
});


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();
    let p = embassy_nrf::init(Default::default());

    let mut temp = Temp::new(p.TEMP, Irqs);

    spawner
        .spawn(button_task(p.P0_14.degrade(), ButtonDirection::Left))
        .unwrap();
    spawner
        .spawn(button_task(p.P0_23.degrade(), ButtonDirection::Right))
        .unwrap();

    let _row1 = led_pin(p.P0_21.degrade());
    let col = [
        led_pin(p.P0_28.degrade()),
        led_pin(p.P0_11.degrade()),
        led_pin(p.P0_31.degrade()),
        led_pin(p.P1_05.degrade()),
        led_pin(p.P0_30.degrade()),
    ];

    let config = twim::Config::default();
    let twim0 = Twim::new(p.TWISPI0, Irqs, p.P0_16, p.P0_08, config);

    // Get sensor
    let mut sensor = Lsm303agr::new_with_i2c(twim0);
    let id = sensor.magnetometer_id().await.unwrap();
    rprintln!("{:#02x?}", id);

    // Initialise Sensor
    sensor.init().await.unwrap();
    sensor
        .set_mag_mode_and_odr(
            &mut Delay,
            lsm303agr::MagMode::HighResolution,
            lsm303agr::MagOutputDataRate::Hz50,
        )
        .await
        .unwrap();

    sensor
        .set_accel_mode_and_odr(
            &mut Delay,
            lsm303agr::AccelMode::HighResolution,
            lsm303agr::AccelOutputDataRate::Hz50,
        )
        .await
        .unwrap();

    let Ok(mut sensor) = sensor.into_mag_continuous().await else {
        panic!("Error enabling continuous mode")
    };

    let mut datastore = DataStore::new(8);

    read_data(&mut datastore, &mut sensor).await;

    let temp_meas = temp.read().await.to_num::<f64>();
    datastore.add_temp(temp_meas);

    // LED task:
    let mut blinker = LedRow::new(col);
    loop {
        blinker.toggle();

        // Read data
        read_data(&mut datastore, &mut sensor).await;
        let temp_meas = temp.read().await.to_num::<f64>();
        datastore.add_temp(temp_meas);

        // Pick up button press or wait
        select_biased! {
            direction = CHANNEL.receive().fuse() => {
                blinker.shift(direction);
            }
            _ = Timer::after_millis(500).fuse() => {
                rprintln!("{:?}", datastore.get_averages());
            }
        }
    }
}

fn led_pin(pin: AnyPin) -> Output<'static> {
    Output::new(pin, Level::High, OutputDrive::Standard)
}


#[embassy_executor::task(pool_size = 2)]
async fn button_task(
    pin: AnyPin,
    direction: ButtonDirection,
) {
    let mut input = Input::new(pin, Pull::None);
    loop {
        input.wait_for_low().await;
        CHANNEL.send(direction).await;
        Timer::after_millis(100).await;
        input.wait_for_high().await;
    }
}

async fn read_data<'a>(
    data: &mut DataStore,
    sensors: &mut SensorType<'a>
) {
    if sensors.mag_status().await.unwrap().xyz_new_data() {
        let magdata = sensors.magnetic_field().await.unwrap();
        data.add_magnetic(magdata);
    }
    if sensors.accel_status().await.unwrap().xyz_new_data() {
        let accdata = sensors.acceleration().await.unwrap();
        data.add_accel(accdata);
    }
}
