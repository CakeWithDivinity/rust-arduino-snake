#![no_std]
#![no_main]

use arduino_hal::{
    clock::MHz16,
    hal::{
        port::{PB2, PB3, PB5, PC3},
        Adc,
    },
    port::{
        mode::{Analog, Output},
        Pin,
    },
};
use max7219::{connectors::PinConnector, MAX7219};
use panic_halt as _;

type Matrix = MAX7219<PinConnector<Pin<Output, PB3>, Pin<Output, PB2>, Pin<Output, PB5>>>;
type Position = (u8, u8);

#[derive(PartialEq)]
enum GameState {
    Start,
    Dead,
    Running,
}

struct GameManager {
    snake: Snake,
    food: Position,
    state: GameState,
}

impl GameManager {
    fn new(analog_noise: &Pin<Analog, PC3>, adc: &mut Adc<MHz16>) -> Self {
        let snake = Snake::new();
        let food = Self::get_next_safe_food_pos(&snake, analog_noise, adc);
        GameManager {
            snake: Snake::new(),
            food,
            state: GameState::Start,
        }
    }

    fn process_movement(&mut self, direction: Option<Direction>) {
        let Some(direction) = direction else {
            return;
        };

        match (&self.snake.direction, &direction) {
            (Direction::Left, Direction::Right)
            | (Direction::Right, Direction::Left)
            | (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up) => (),
            _ => self.snake.direction = direction,
        }
    }

    fn tick(&mut self, analog_noise: &Pin<Analog, PC3>, adc: &mut Adc<MHz16>) {
        let next_pos = self.snake.get_next_pos();
        if self.snake.is_on_pos(next_pos) || is_position_out_of_bounds(next_pos) {
            self.state = GameState::Dead;
            return;
        }

        let snake_tail = self.snake.tick();
        let snake_head = self.snake.body[0];

        if snake_head == self.food {
            self.snake.body[self.snake.length as usize] = snake_tail;
            self.snake.length += 1;

            self.food = Self::get_next_safe_food_pos(&self.snake, analog_noise, adc);
        }
    }

    fn get_next_safe_food_pos(
        snake: &Snake,
        analog_noise: &Pin<Analog, PC3>,
        adc: &mut Adc<MHz16>,
    ) -> Position {
        loop {
            let food = generate_rand_food_pos(analog_noise, adc);

            if !snake.is_on_pos(food) {
                return food;
            }
        }
    }

    fn to_image(&self) -> [u8; 8] {
        let mut image: [u8; 8] = [0; 8];

        for i in 0..self.snake.length {
            let body_part = self.snake.body[i as usize];
            let column: usize = (body_part.0 - 1).into();
            let row = body_part.1 - 1;

            image[column] |= 0b1 << row;
        }

        image[(self.food.0 - 1) as usize] |= 0b1 << (self.food.1 - 1);
        image
    }
}

struct Snake {
    body: [Position; 64],
    direction: Direction,
    length: u8,
}

impl Snake {
    fn new() -> Self {
        let mut body = [(0, 0); 64];
        body[0] = (5, 5);
        body[1] = (4, 5);
        body[2] = (3, 5);

        Self {
            body,
            direction: Direction::Right,
            length: 3,
        }
    }

    fn tick(&mut self) -> (u8, u8) {
        let mut previous = self.body[0];

        let next_pos = self.get_next_pos();
        self.body[0] = next_pos;

        for i in 1..self.length {
            core::mem::swap(&mut self.body[i as usize], &mut previous);
        }

        previous
    }

    fn is_on_pos(&self, pos: (u8, u8)) -> bool {
        self.body.iter().any(|snake_part| *snake_part == pos)
    }

    fn get_next_pos(&self) -> (u8, u8) {
        let movement = self.direction.to_index_movement();
        let head = self.body[0];
        (
            head.0.wrapping_add_signed(movement.0),
            head.1.wrapping_add_signed(movement.1),
        )
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
            Self::Right => (1, 0),
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let joy_x = pins.a1.into_analog_input(&mut adc);
    let joy_y = pins.a2.into_analog_input(&mut adc);
    let joy_button = pins.d2.into_pull_up_input();

    let analog_noise = pins.a3.into_analog_input(&mut adc);

    let mut max = MAX7219::from_pins(
        1,
        pins.d11.into_output(),
        pins.d10.into_output(),
        pins.d13.into_output(),
    )
    .unwrap();
    max.power_on().unwrap();

    let mut gamestate = GameManager::new(&analog_noise, &mut adc);
    write_image(&mut max, gamestate.to_image());

    loop {
        if gamestate.state != GameState::Running {
            if joy_button.is_low() {
                if gamestate.state == GameState::Dead {
                    gamestate = GameManager::new(&analog_noise, &mut adc);
                }

                gamestate.state = GameState::Running;
            }

            continue;
        }

        let new_direction = match (joy_x.analog_read(&mut adc), joy_y.analog_read(&mut adc)) {
            (x, _) if x > 900 => Some(Direction::Right),
            (x, _) if x < 100 => Some(Direction::Left),
            (_, y) if y > 900 => Some(Direction::Down),
            (_, y) if y < 100 => Some(Direction::Up),
            _ => None,
        };

        gamestate.process_movement(new_direction);
        gamestate.tick(&analog_noise, &mut adc);

        write_image(&mut max, gamestate.to_image());
        arduino_hal::delay_ms(500);
    }
}

fn write_image(max: &mut Matrix, image: [u8; 8]) {
    for column in 0..8 {
        max.write_raw_byte(0, column + 1, image[column as usize])
            .unwrap();
    }
}

fn generate_rand_food_pos(analog_noise: &Pin<Analog, PC3>, adc: &mut Adc<MHz16>) -> Position {
    (
        (analog_noise.analog_read(adc) % 8 + 1) as u8,
        (analog_noise.analog_read(adc) % 8 + 1) as u8,
    )
}

fn is_position_out_of_bounds(pos: Position) -> bool {
    pos.0 == 0 || pos.0 > 8 || pos.1 == 0 || pos.1 > 8
}
