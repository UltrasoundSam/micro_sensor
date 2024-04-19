#![no_std]
#![no_main]

use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use panic_halt as _;
use heapless::Vec;

use microbit::{
    hal::{twim::Twim, uarte, Delay, Rtc},
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{
    AccelMode, AccelOutputDataRate, Lsm303agr, Acceleration
};

mod serial_comms;
mod control;

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

    // Setup sensor
    let mut imu = Lsm303agr::new_with_i2c(i2c);
    imu.init().unwrap();
    imu.set_accel_mode_and_odr(&mut delay, AccelMode::HighResolution, AccelOutputDataRate::Hz50).unwrap();

    // Setup buttons
    control::init_buttons(board.GPIOTE, board.buttons);

    // Create Struct to hold average values
    let num_averages = control::get_num_aves();
    let mut aves = SimpleMovingAverage::new(num_averages);

    if imu.accel_status().unwrap().xyz_new_data() {
        // Get current time
        current_time = timer.get_counter();
        diff = (current_time - previous_time) as f64 / clock_freq;
        previous_time = current_time;
        elapsed_time += diff;

        // Read data
        let data = imu.acceleration().unwrap();
        aves.add_measurement(data);

        serial.send_data(aves.get_average(), elapsed_time);
        rprintln!("x: {}, y: {}, z {}", data.x_mg(), data.y_mg(), data.z_mg());
    }

    // Update time just before loop
    current_time = timer.get_counter();
    diff = (current_time - previous_time) as f64 / clock_freq;
    previous_time = current_time;
    elapsed_time += diff;
    loop {
        // Check if data is available
        if imu.accel_status().unwrap().xyz_new_data() {
            // If it is, let's take a measurement
            let data = imu.acceleration().unwrap();
            aves.add_measurement(data);
        }

        current_time = timer.get_counter();
        diff = (current_time - previous_time) as f64 / clock_freq;
        if diff >= interval as f64 {
            // Update timings
            timer.clear_counter();
            previous_time = 0;
            elapsed_time += diff;

            if control::get_meas_state() {
                // Read data
                let ave_data = aves.get_average();

                // Send data
                serial.send_data(ave_data, elapsed_time);

                // Create new averages
                let num_averages = control::get_num_aves();
                rprintln!("{}",num_averages);
                aves.update_size(num_averages);
            }
        }
    }
}

struct SimpleMovingAverage {
    number: u8,
    acc_x: Vec<i32, 255>,
    acc_y: Vec<i32, 255>,
    acc_z: Vec<i32, 255>,
}

impl SimpleMovingAverage {
    fn new(size: u8) -> SimpleMovingAverage {
        SimpleMovingAverage {
            number: size,
            acc_x: Vec::new(),
            acc_y: Vec::new(),
            acc_z: Vec::new()
        }
    }

    fn add_measurement(&mut self, measurement: Acceleration) {
        // Get values
        let values = measurement.xyz_mg();

        // Push values onto vec
        self.acc_x.push(values.0).unwrap();
        self.acc_y.push(values.1).unwrap();
        self.acc_z.push(values.2).unwrap();

        // Check to see if we have exceeded number of averages
        if self.acc_x.len() as u8 > self.number {
            self.acc_x.remove(0);
            self.acc_y.remove(0);
            self.acc_z.remove(0);
        }
    }

    fn update_size(&mut self, new_size: u8) {
        self.number = new_size;
    }

    fn get_average(&self) -> (f64, f64, f64) {
        let sum_x = self.acc_x.iter().fold(0, |acc, x| acc+x);
        let sum_y = self.acc_y.iter().fold(0, |acc, x| acc+x);
        let sum_z = self.acc_z.iter().fold(0, |acc, x| acc+x);

        let num_elems = self.acc_x.len() as f64;
        let ave_x = sum_x as f64 / num_elems;
        let ave_y = sum_y as f64 / num_elems;
        let ave_z = sum_z as f64 / num_elems;
        let result = (ave_x, ave_y, ave_z);
        result
    }
}