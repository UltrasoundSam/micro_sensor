#![no_std]
#![no_main]

use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use panic_halt as _;

use microbit::{
    hal::{twim::Twim, uarte, Delay, Rtc},
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{
    AccelMode, AccelOutputDataRate, Lsm303agr, MagMode, MagOutputDataRate,
};

mod serial_comms;
mod control;
mod average;

#[entry]
fn main() -> ! {
    // Setup board
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    // Define prescaler and frequency of clock
    let prescaler: u32 = 0;
    let clock_freq = 32_768f64 / (prescaler as f64 + 1.);

    // Define counter to see how many milliseconds have passed.
    let mut current_time: u32 = 0;
    let mut previous_time : u32 = 0;
    let mut elapsed_time: f64 = 0.;
    let mut diff = (current_time - previous_time) as f64 / clock_freq;
    elapsed_time += diff;

    // Define delay (in seconds)
    let interval = 1.;

    // Creating timer for IMU and for timestamp
    let mut delay = Delay::new(board.SYST);
    let timer = Rtc::new(board.RTC0, prescaler).unwrap();
    timer.enable_counter();

    // Setup Serial Connection
    let serial = uarte::Uarte::new(
        board.UARTE0,
        board.uart.into(),
        uarte::Parity::EXCLUDED,
        uarte::Baudrate::BAUD115200);
    let mut serial = serial_comms::UartePort::new(serial);

    // Set up i2c
    serial.write_str("Setting up i2c and imu interface...").unwrap();
    let i2c =  Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);

    // Setup accelerometer and magnetometer sensors
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_mode_and_odr(&mut delay, AccelMode::HighResolution, AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_mode_and_odr(&mut delay, MagMode::HighResolution, MagOutputDataRate::Hz50).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    // Setup buttons
    control::init_buttons(board.GPIOTE, board.buttons);

    // Create Struct to hold average values
    let num_averages = control::get_num_aves();
    let mut aves = average::SimpleMovingAverage::new(num_averages);

    if sensor.accel_status().unwrap().xyz_new_data() {
        // Get current time
        current_time = timer.get_counter();
        diff = (current_time - previous_time) as f64 / clock_freq;
        previous_time = current_time;
        elapsed_time += diff;

        // Read data
        let acc_data = sensor.acceleration().unwrap();
        let mag_data = sensor.magnetic_field().unwrap();
        aves.add_acceleration(acc_data);
        aves.add_magnetic(mag_data);

        serial.send_data(elapsed_time, &aves);
        rprintln!("x: {}, y: {}, z {}", acc_data.x_mg(), acc_data.y_mg(), acc_data.z_mg());
    }

    // Update time just before loop
    current_time = timer.get_counter();
    diff = (current_time - previous_time) as f64 / clock_freq;
    previous_time = current_time;
    elapsed_time += diff;
    loop {
        // Check if acceleration data is available
        if sensor.accel_status().unwrap().xyz_new_data() {
            // If it is, let's take a measurement
            let acc_data = sensor.acceleration().unwrap();
            aves.add_acceleration(acc_data);
        }

        // Check if magnetic field data is available
        if sensor.mag_status().unwrap().xyz_new_data() {
            // If it iss, let's take a measurement
            let mag_data = sensor.magnetic_field().unwrap();
            aves.add_magnetic(mag_data);
        }

        current_time = timer.get_counter();
        diff = (current_time - previous_time) as f64 / clock_freq;
        if diff >= interval as f64 {
            // Update timings
            timer.clear_counter();
            previous_time = 0;
            elapsed_time += diff;

            if control::get_meas_state() {
                // Send data
                serial.send_data(elapsed_time, &aves);

                // Create new averages
                let num_averages = control::get_num_aves();
                rprintln!("{}",num_averages);
                aves.update_size(num_averages);
            }
        }
    }
}
