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

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);


    let mut max = MAX7219::from_pins(1, pins.d11.into_output(), pins.d10.into_output(), pins.d13.into_output()).unwrap();
    max.power_on().unwrap();

    let gamestate = GameState::new();

    loop {
        write_image(&mut max, gamestate.to_image());
    }
}

fn write_image(max: &mut Matrix, image: [u8; 8]) {
    for column in 0..8 {
        max.write_raw_byte(0, column + 1, image[column as usize]).unwrap();
    }
}
