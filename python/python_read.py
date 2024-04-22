import serial

import microbit_serial


def main():
    # Setup socket
    ser = serial.Serial('COM6', baudrate=115200)

    # Just continually read data
    while True:
        res = microbit_serial.read_packet(ser)
        print(res)


if __name__ == "__main__":
    main()
