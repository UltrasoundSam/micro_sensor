"""
Some useful helper functions for reading and parsing data from microbit
"""
import serial
import struct


def parse_packet(conn: serial.Serial) -> tuple[float, int, int, int] | None:
    ''' Reads from socket and formats packet correctly.

    Reads information from serial port, and parses the packet correctly,
    given the structure of the packet given below.

    Parameters:
        conn (Serial): an open Serial object

    Returns:
        packet (tuple): Data packet tuple (see below)


    NOTE: Data packet tuple is expected to be in the form of
    (   f64,       u8,     f64,    f64,   f64,  f64,   f64,   f64)
    (timestamp, num_aves, acc_x, acc_y, acc_z, mag_x, mag_y, mag_z)
    '''
    # Read in packet
    packet = read_packet(conn)

    # Now we have a correctly formatted packet, we can unpack it
    # as (f64, u8, f64, f64, f64, f64, f64, f64)
    try:
        # d - double (f64), B - unsigned char, c - char.
        struct_fmt = '>dB6d'
        result = struct.unpack(struct_fmt, packet)
    except struct.error:
        # Just going to be setup message at the start,
        # can just ignore it for now
        return

    # Discard last two items (\r\n)
    packet = result[:-2]
    return packet


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
