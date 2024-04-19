import serial
import struct


def read_packet(conn: serial.Serial) -> tuple[float, int, int, int]:
    ''' Reads from socket and formats packet correctly.

    Reads information from serial port, and parses the packet correctly,
    given the structure of the packet (timestamp, acc_x, acc_y, acc_z),
    which has the type structure (double, int, int, int)

    Parameters:
        conn (Serial): an open Serial object

    Returns:
        packet (tuple): Data packet tuple (timestamp, num_aves, acc_x,
                                           acc_y, acc_z)
    '''
    buff = bytearray()
    while True:
        # Check to see if message is terminated
        if buff.endswith(b'\r\n'):
            break

        # Read next byte into buffer
        buff.extend(conn.read())

    # Now we have a correctly formatted packet, we can unpack it
    # as (f64, u8, f64, f64, f64, char[u8], char[u8])
    try:
        # d - double (f64), B - unsigned char, c - char.
        struct_fmt = '>dB3d2c'
        result = struct.unpack(struct_fmt, buff)
    except struct.error:
        # Just going to be setup message at the start,
        # can just ignore it for now
        return

    # Discard last two items (\r\n)
    packet = result[:-2]
    return packet


def main():
    # Setup socket
    ser = serial.Serial('COM6', baudrate=115200)

    # Just continually read data
    while True:
        res = read_packet(ser)
        print(res)


if __name__ == "__main__":
    main()
