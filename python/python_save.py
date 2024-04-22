import datetime as dt
import serial

import microbit_serial


def main():
    # Setup socket
    ser = serial.Serial('COM6', baudrate=115200)

    # Read first packet to see when first data packet is received
    res = microbit_serial.read_packet(ser)
    time_now = dt.datetime.now(dt.timezone.utc)

    # Create filename (Windows doesn't like :
    filename = rf"./{time_now.isoformat().replace(':', '_')}_Microbit.bin"

    # Save data to file
    with open(filename, 'wb') as fi:
        # Just continually read & save data
        for _ in range(10):
            res = microbit_serial.read_packet(ser)
            fi.write(res)

    # Periodically write data every 60 samples
    while True:
        with open(filename, 'ab') as fi:
            for _ in range(60):
                res = microbit_serial.read_packet(ser)
                fi.write(res)


if __name__ == "__main__":
    main()
