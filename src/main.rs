#![no_std]
#![no_main]


use arduino_hal::{port::{mode::Output, Pin}, hal::port::{PB3, PB2, PB5}};
use max7219::{MAX7219, connectors::PinConnector};
use panic_halt as _;

type Matrix = MAX7219<PinConnector<Pin<Output, PB3>, Pin<Output, PB2>, Pin<Output, PB5>>>;

struct GameState {
    snake: Snake,
}

impl GameState {
    fn new() -> Self {
        GameState { snake: Snake::new() }
    }

    fn to_image(&self) -> [u8; 8] {
        let mut image: [u8; 8] = [0; 8];

        for i in 0..self.snake.length {
            let body_part = self.snake.body[i as usize];
            let column: usize = (body_part.0 - 1).into();
            let row: usize = (body_part.1 - 1).into();

            image[column] = image[column] | (0b1 << row);
        }

        image
    }
}

struct Snake {
    body: [(u8, u8); 64],
    length: u8,
}

impl Snake {
    fn new() -> Self {
            let mut body = [(0, 0); 64];
            body[0] = (5, 5);
            body[1] = (5, 6);
            body[2] = (6, 6);

        Self {
            body,
            length: 3,
        }
    }
}

const IMAGE_LEFT: [u8; 8] = [0b10000, 0, 0, 0, 0, 0, 0, 0];
const IMAGE_RIGHT: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0b10000];
const IMAGE_UP: [u8; 8] = [0, 0, 0, 0, 0b1, 0, 0, 0];
const IMAGE_DOWN: [u8; 8] = [0, 0, 0, 0, 0b10000000, 0, 0, 0];

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let joy_x = pins.a1.into_analog_input(&mut adc);
    let joy_y = pins.a2.into_analog_input(&mut adc);

    let mut max = MAX7219::from_pins(1, pins.d11.into_output(), pins.d10.into_output(), pins.d13.into_output()).unwrap();
    max.power_on().unwrap();

    let gamestate = GameState::new();

    loop {

        match (joy_x.analog_read(&mut adc), joy_y.analog_read(&mut adc)) {
            (x, _) if x > 1000 => write_image(&mut max, IMAGE_RIGHT),
            (x, _) if x < 100 => write_image(&mut max, IMAGE_LEFT),
            (_, y) if y > 1000 => write_image(&mut max, IMAGE_DOWN),
            (_, y) if y < 100 => write_image(&mut max, IMAGE_UP),
            _ => write_image(&mut max, [0; 8]),
        }

        // write_image(&mut max, gamestate.to_image());
    }
}

fn write_image(max: &mut Matrix, image: [u8; 8]) {
    for column in 0..8 {
        max.write_raw_byte(0, column + 1, image[column as usize]).unwrap();
    }
}
