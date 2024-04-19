use core::fmt::{Error, Write};
use microbit::hal::uarte::{Uarte, Instance};

use super::average;

// pub struct UartePort<T: Instance>(Uarte<T>);
pub struct UartePort<T: Instance> {
    pub conn: Uarte<T>,
}

impl<T: Instance> UartePort<T> {
    pub fn new(conn: Uarte<T>) -> Self {
        UartePort { conn }
    }

    pub fn send_data(&mut self, timestamp: f64, ave_struct: &average::SimpleMovingAverage) {
        // Send timestamp data
        self.conn.write(&timestamp.to_be_bytes()).unwrap();

        // Unpack structure
        let averages = ave_struct.get_num_aves();
        let acc_aves = ave_struct.get_acc_average();
        let mag_aves = ave_struct.get_mag_average();

        // Send bytes
        self.conn.write(&averages.to_be_bytes()).unwrap();
        self.conn.write(&acc_aves.0.to_be_bytes()).unwrap();
        self.conn.write(&acc_aves.1.to_be_bytes()).unwrap();
        self.conn.write(&acc_aves.2.to_be_bytes()).unwrap();
        self.conn.write(&mag_aves.0.to_be_bytes()).unwrap();
        self.conn.write(&mag_aves.1.to_be_bytes()).unwrap();
        self.conn.write(&mag_aves.2.to_be_bytes()).unwrap();
        self.conn.write_str("\r\n").unwrap();
    }

    pub fn write_str(&mut self, msg: &str) -> Result<(), Error>{
        self.conn.write_str(msg)
    }
}
