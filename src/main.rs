#![no_std]
#![no_main]

use max7219::MAX7219;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);


    let mut max = MAX7219::from_pins(1, pins.d11.into_output(), pins.d10.into_output(), pins.d13.into_output()).unwrap();

    max.power_on().unwrap();
    max.write_raw_byte(0, 0b0010, 0b0010).unwrap();
    loop {
    }
}
