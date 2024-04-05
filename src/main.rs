#![no_std]
#![no_main]
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rtt_init_print, rprintln};

use microbit::{
    hal::{twim::Twim, Delay},
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{
    AccelMode, AccelOutputDataRate, Lsm303agr
};


#[entry]
fn main() -> ! {
    // Setup board
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    // Creating timer for IMU
    let mut delay = Delay::new(board.SYST);

    // Set up i2c
    let i2c =  Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);

    // Setup sensor
    let mut imu = Lsm303agr::new_with_i2c(i2c);
    imu.init().unwrap();
    imu.set_accel_mode_and_odr(&mut delay, AccelMode::HighResolution, AccelOutputDataRate::Hz50).unwrap();

    loop {
        if imu.accel_status().unwrap().xyz_new_data() {
            let data = imu.acceleration().unwrap();
            rprintln!("Acceleration: x {} y {} z {}", data.x_mg(), data.y_mg(), data.z_mg());
        }
    }
}
