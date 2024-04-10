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
    AccelMode, AccelOutputDataRate, Lsm303agr
};
use core::fmt::Write;

mod serial_comms;

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
    let mut serial = uarte::Uarte::new(
        board.UARTE0,
        board.uart.into(),
        uarte::Parity::EXCLUDED,
        uarte::Baudrate::BAUD115200);

    // Set up i2c
    writeln!(serial, "Setting up i2c and imu interface...").unwrap();
    let i2c =  Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);

    // Setup sensor
    let mut imu = Lsm303agr::new_with_i2c(i2c);
    imu.init().unwrap();
    imu.set_accel_mode_and_odr(&mut delay, AccelMode::HighResolution, AccelOutputDataRate::Hz50).unwrap();

    if imu.accel_status().unwrap().xyz_new_data() {
        // Get current time
        current_time = timer.get_counter();
        diff = (current_time - previous_time) as f64 / clock_freq;
        previous_time = current_time;
        elapsed_time += diff;

        // Read data
        let data = imu.acceleration().unwrap();

        serial_comms::send_data(&data, &mut serial, &elapsed_time);
        rprintln!("x: {}, y: {}, z {}", data.x_mg(), data.y_mg(), data.z_mg());
    }

    // Update time just before loop
    current_time = timer.get_counter();
    diff = (current_time - previous_time) as f64 / clock_freq;
    previous_time = current_time;
    elapsed_time += diff;
    loop {
        current_time = timer.get_counter();
        diff = (current_time - previous_time) as f64 / clock_freq;
        if diff >= interval as f64 {
            // Update timings
            timer.clear_counter();
            previous_time = 0;
            elapsed_time += diff;

            // Read data
            let data = imu.acceleration().unwrap();

            // Send data
            serial_comms::send_data(&data, &mut serial, &elapsed_time);
            rprintln!("{}", elapsed_time);
        }

        // Check to see whether new message has arrived via uart, and parse
        // if serial.
    }
}
