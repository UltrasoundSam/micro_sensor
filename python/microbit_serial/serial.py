"""
Some useful helper functions for reading and parsing data from microbit
"""
import serial
import struct

data_packet = tuple[float,
                    int,
                    float, float, float,
                    float, float, float,
                    float]


def parse_packet(conn: serial.Serial) -> data_packet | None:
    ''' Reads from socket and formats packet correctly.

    Reads information from serial port, and parses the packet correctly,
    given the structure of the packet given below.

    Parameters:
        conn (Serial): an open Serial object

    Returns:
        packet (tuple): Data packet tuple (see below)


    NOTE: Data packet tuple is expected to be in the form of
    (   f64,       u8,     f64,   f64,   f64,   f64,   f64,   f64,  f64 )
    (timestamp, num_aves, acc_x, acc_y, acc_z, mag_x, mag_y, mag_z, temp)
    '''
    # Read in packet
    packet = read_packet(conn)

    result = unpack_packet(packet)
    return result


def read_packet(conn: serial.Serial) -> bytearray:
    ''' Reads bytes from socket and returns latest data packet.

    Reads information from serial port, and parses the packet correctly,
    given the structure of the packet (timestamp, acc_x, acc_y, acc_z),
    which has the type structure (double, int, int, int)

    Parameters:
        conn (Serial): an open Serial object

    Returns:
        packet (tuple): Data packet tuple (see below)
    '''
    buff = bytearray()
    while True:
        # Check to see if message is terminated
        if buff.endswith(b'\r\n'):
            break

        # Read next byte into buffer
        buff.extend(conn.read())

    # Discard last two items (\r\n)
    packet = buff[:-2]
    return packet


def unpack_packet(msg: bytearray) -> data_packet | None:
    '''Unpacks the message

    Given the data packet tuple is expected to be in the form of
    (   f64,       u8,     f64,    f64,   f64,  f64,   f64,   f64,  f64 )
    (timestamp, num_aves, acc_x, acc_y, acc_z, mag_x, mag_y, mag_z, temp)

    Input:
        msg - series of bytes to unpack

  Returns:
        result (tuple): Data packet tuple (see below) or None if not valid


    NOTE: Data packet tuple is expected to be in the form of
    (   f64,       u8,     f64,    f64,   f64,  f64,   f64,   f64,  f64 )
    (timestamp, num_aves, acc_x, acc_y, acc_z, mag_x, mag_y, mag_z, temp)
    '''
    # Unpack msg as (f64, u8, f64, f64, f64, f64, f64, f64, f64)
    try:
        # d - double (f64), B - unsigned char, c - char.
        struct_fmt = '>dB7d'
        result = struct.unpack(struct_fmt, msg)
    except struct.error:
        # Just going to be setup message at the start,
        # can just ignore it for now
        return

    # Discard last two items (\r\n)
    result = result[:-2]
    return result
