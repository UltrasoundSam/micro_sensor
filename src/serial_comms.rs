use lsm303agr::Acceleration;
use microbit::{
    pac::UARTE0,
    hal::uarte,
};
use heapless::Vec;
use core::fmt::Write;
use core::str::from_utf8;

/// Function for sending measurement from Lsm303agr over UART serial connection.
///
/// Takes in an Lsm303agr::Acceleration struct, breaks out the scaled
/// x, y and x values for the acceleration and sends them over the serial connection.
/// # Inputs
/// measurement - reference to Acceleration struct, to read values
/// conn - mutable reference to serial connection to send data
///
/// # Returns
/// - None
pub fn send_data(measurement: &Acceleration, conn: &mut uarte::Uarte<UARTE0>, timestamp: &f64) {
    // Send timestamp bytes
    conn.write(&timestamp.to_be_bytes()).unwrap();
    // Get values
    let values = measurement.xyz_mg();

    // Send bytes
    conn.write(&values.0.to_be_bytes()).unwrap();
    conn.write(&values.1.to_be_bytes()).unwrap();
    conn.write(&values.2.to_be_bytes()).unwrap();
    write!(conn, "\r\n").unwrap();
}

/// Function for parsing incoming serial data
///
/// Reads from given serial connection and parses message.
fn read_data(conn: &mut uarte::Uarte<UARTE0>) -> Vec<u8, 32> {
    // Create RX Buffer and initiate to zero
    let mut rx_buf: [u8; 1] = [0; 1];
    // Create buffer - 32 chars long
    let mut buffer: Vec<u8, 32> = Vec::new();

    // Loop through until we find a delimiter
    loop {
        conn.read(&mut rx_buf).unwrap();

        // Push byte onto vector and check for error
        if buffer.push(rx_buf[0]).is_err() {
            write!(conn, "Error: Buffer is full\r\n").unwrap();
            break;
        }

        // Check for delimiter
        if rx_buf[0] == b'\n' {
            break;
        }
    }
    buffer
}

/// Parses incoming serial data
///
/// Reads data from serial connection, checks info is correct and returns result
pub fn parse_data(conn: &mut uarte::Uarte<UARTE0>) -> f64 {
    // Read in data
    let buff = read_data(conn);

    // Set default value
    let mut result = 0.;

    // Check to see whether message is correct
    let msg_len = buff.len();
    let cmd_msg = from_utf8(&buff[..4]).unwrap();
    if (msg_len == 12) & (cmd_msg == "frq:") {
        result = f64::from_be_bytes(buff[4..12].try_into().unwrap());
    }

    result
}
