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

    fn process_movement(&mut self, direction: Option<Direction>) {
        let Some(direction) = direction else {
            return;
        };

        match (&self.snake.direction, &direction) {
            | (Direction::Left, Direction::Right)
            | (Direction::Right, Direction::Left)
            | (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up) => (),
            _ => self.snake.direction = direction
        }
    }

    fn tick(&mut self) {
        self.snake.tick();
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
    direction: Direction,
    length: u8,
}

impl Snake {
    fn new() -> Self {
        let mut body = [(0, 0); 64];
        body[0] = (5, 5);

        Self {
            body,
            direction: Direction::Right,
            length: 1,
        }
    }

    fn tick(&mut self) {
        let mut previous = self.body[0].clone();

        let movement = self.direction.to_index_movement();
        self.body[0] = (previous.0.wrapping_add_signed(movement.0), previous.1.wrapping_add_signed(movement.1));

        if self.length < 1 {
            return;
        }

        for i in 1..self.length {
            let new_prev = self.body[i as usize];
            self.body[i as usize] = previous;
            previous = new_prev;
        }
    }
}

#[derive(Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn to_index_movement(&self) -> (i8, i8) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0)
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

    let mut gamestate = GameState::new();

    loop {

        let new_direction = match (joy_x.analog_read(&mut adc), joy_y.analog_read(&mut adc)) {
            (x, _) if x > 1000 => Some(Direction::Right),
            (x, _) if x < 100 => Some(Direction::Left),
            (_, y) if y > 1000 => Some(Direction::Down),
            (_, y) if y < 100 => Some(Direction::Up),
            _ => None, 
        };

        gamestate.process_movement(new_direction);
        gamestate.tick();

        write_image(&mut max, gamestate.to_image());
        arduino_hal::delay_ms(1000);
    }
}

fn write_image(max: &mut Matrix, image: [u8; 8]) {
    for column in 0..8 {
        max.write_raw_byte(0, column + 1, image[column as usize]).unwrap();
    }
}
