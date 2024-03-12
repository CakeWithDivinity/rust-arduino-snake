# rust-arduino-snake

An implementation of Snake for the Arduino Uno in Rust using avr-hal.

## Build Instructions

1. Install prerequisites as described in the [avr-hal README](https://github.com/Rahix/avr-hal) (`avr-gcc`, `avr-libc`, `avrdude`, `ravedude`).

2. Run `cargo build` to build the firmware.

3. Add the path for the usb connection to your Arduino as an environment variable. For example:

```bash
export RAVEDUDE_PORT=/dev/ttyACM0
```

3. Run `cargo run` to flash the firmware to a connected board. If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

## Arduino Setup

I am using a MAX7129 controller connected to a 8x8 LED matrix <https://amzn.eu/d/1LI1OAG> and an analog joystick <https://amzn.eu/d/6GT864N>

The setup looks like this:
![alt text](https://github.com/CakeWithDivinity/rust-arduino-snake/blob/main/arduino_setup.jpg?raw=true)

## Contribution

This was my first time doing something with an Arduino and with embedded Rust. So feel free to leave any feedback as an issue
or submit PRs
