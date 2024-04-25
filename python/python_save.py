from zoneinfo import ZoneInfo
import datetime as dt
import serial

import microbit_serial


def read_and_check(connection: serial.Serial) -> bytes:
    '''Checks that the received data is a valid data packet

    Attempts to parse the data packet - and discards it if
    it cannot be unpacked.

    Inputs:
        connection (Serial object) - Serial object to read from

    Returns:
        valid_packet (bytes) - valid packet of 57 bytes
    '''
    msg = microbit_serial.read_packet(connection)

    # Returns None if not a valid packet
    valid_packet = microbit_serial.unpack_packet(msg)

    if valid_packet:
        return msg


def main():
    # Setup socket
    ser = serial.Serial('COM6', baudrate=115200)

    # Read first packet to see when first data packet is received
    res = microbit_serial.read_packet(ser)
    time_now = dt.datetime.now(ZoneInfo("Europe/London"))

    # Create filename (Windows doesn't like :
    filename = rf"./{time_now.isoformat().replace(':', '_')}_Microbit.bin"

    # Save data to file
    with open(filename, 'wb') as fi:
        # Just continually read & save data
        for _ in range(10):
            res = read_and_check(ser)
            try:
                fi.write(res)
            except TypeError:
                # Read and Check returned None - so not valid packet
                continue

    # Periodically write data every 60 samples
    while True:
        with open(filename, 'ab') as fi:
            for _ in range(60):
                res = read_and_check(ser)
                try:
                    fi.write(res)
                except TypeError:
                    # Read and Check returned None - so not valid packet
                    continue


if __name__ == "__main__":
    main()
