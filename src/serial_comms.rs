use core::fmt::{Error, Write};
use microbit::hal::uarte::{Uarte, Instance};

// pub struct UartePort<T: Instance>(Uarte<T>);
pub struct UartePort<T: Instance> {
    pub conn: Uarte<T>,
}

impl<T: Instance> UartePort<T> {
    pub fn new(conn: Uarte<T>) -> Self {
        UartePort { conn }
    }

    pub fn send_data(&mut self, measurement: (f64, f64, f64),
        timestamp: f64, averages: u8) {
        // Send timestamp data
        self.conn.write(&timestamp.to_be_bytes()).unwrap();

        // Send bytes
        self.conn.write(&averages.to_be_bytes()).unwrap();
        self.conn.write(&measurement.0.to_be_bytes()).unwrap();
        self.conn.write(&measurement.1.to_be_bytes()).unwrap();
        self.conn.write(&measurement.2.to_be_bytes()).unwrap();
        self.conn.write_str("\r\n").unwrap();
    }

    pub fn write_str(&mut self, msg: &str) -> Result<(), Error>{
        self.conn.write_str(msg)
    }
}
