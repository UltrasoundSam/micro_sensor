use core::fmt::{Error, Write};
use microbit::hal::uarte::{Uarte, Instance};

use lsm303agr::Acceleration;

// pub struct UartePort<T: Instance>(Uarte<T>);
pub struct UartePort<T: Instance> {
    pub conn: Uarte<T>,
}

impl<T: Instance> UartePort<T> {

    pub fn send_data(&mut self, measurement: Acceleration, timestamp: f64) {
        // Send timestamp data
        self.conn.write(&timestamp.to_be_bytes()).unwrap();

        // Get values
        let values = measurement.xyz_mg();

        // Send bytes
        self.conn.write(&values.0.to_be_bytes()).unwrap();
        self.conn.write(&values.1.to_be_bytes()).unwrap();
        self.conn.write(&values.2.to_be_bytes()).unwrap();
        self.conn.write_str("\r\n").unwrap();
    }

    pub fn write_str(&mut self, msg: &str) -> Result<(), Error>{
        self.conn.write_str(msg)
    }


}
