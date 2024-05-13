use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::io::{stdout, Stdout, Write};
use std::{thread, time::Duration};

use rand::Rng;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, color, cursor, terminal_size, AsyncReader};

#[derive(Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct UserInput {
    input: termion::input::Keys<AsyncReader>,
}

struct Output {
    output: termion::raw::RawTerminal<Stdout>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct GridCell {
    x: u16,
    y: u16,
}

struct Game {
    grid: HashSet<GridCell>,
    snake: VecDeque<GridCell>,
    food: GridCell,
    direction: Direction,
    input: UserInput,
    output: Output,
    min_width: u16,
    min_height: u16,
    max_width: u16,
    max_height: u16,
}

impl Game {
    fn initialize(&mut self) {
        // Initialize game grid
        for i in self.min_width..self.max_width {
            for j in self.min_height..self.max_height {
                self.grid.insert(GridCell { x: i, y: j });
            }
        }

        // Initialize snake
        let init_size = 5;
        for i in 1..init_size + 1 {
            self.snake.push_front(GridCell {
                x: self.max_width - i,
                y: self.max_height / 2,
            });
        }

        // Update grid by removing cells occupied by snake
        for seg in &self.snake {
            self.grid.remove(&seg);
        }

        // Generate food in a random cell
        let grid_list = self.grid.iter().cloned().collect();
        self.generate_random_food(grid_list);
    }

    fn generate_random_food(&mut self, grid_list: Vec<GridCell>) {
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..grid_list.len());
        self.food = self.grid.take(&grid_list[random_index]).unwrap();
    }

    fn move_snake(&mut self) {
        // Get current head
        let head = self.snake.front().unwrap();

        // Create new head based on direction
        let new_head = match self.direction {
            Direction::Right if head.x == self.max_width => GridCell { x: 1, y: head.y },
            Direction::Right => GridCell {
                x: head.x + 1,
                y: head.y,
            },
            Direction::Left if head.x == 1 => GridCell {
                x: self.max_width,
                y: head.y,
            },
            Direction::Left => GridCell {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Up if head.y == 0 => GridCell {
                x: head.x,
                y: self.max_height,
            },
            Direction::Up => GridCell {
                x: head.x,
                y: head.y - 1,
            },
            Direction::Down if head.y == self.max_height => GridCell { x: head.x, y: 1 },
            Direction::Down => GridCell {
                x: head.x,
                y: head.y + 1,
            },
        };

        // Push new head to start of snake
        self.snake.push_front(new_head);

        // Remove old tail from snake
        self.snake.pop_back();
    }

    fn check_collision(&mut self) -> bool {
        let mut collision = false;
        let head = self.snake.front().unwrap();

        for segment in self.snake.range(1..) {
            if head == segment {
                collision = true;
                break;
            }
        }
        collision
    }

    fn render_snake(&mut self) {
        let mut segments: usize = 0;
        let len = self.snake.len();
        for segment in &self.snake {
            let segment_char = match segments {
                0 => 'S',
                num if num == len - 1 => 'e',
                1 => 'n',
                num if num == len - 2 => 'k',
                _ => 'a',
            };
            write!(
                self.output.output,
                "{goto}{bgColor}{segment_char}{hide}{reset}",
                goto = cursor::Goto(segment.x, segment.y),
                bgColor = color::Bg(color::Green),
                segment_char = segment_char,
                hide = cursor::Hide,
                reset = color::Bg(color::Reset),
            )
            .unwrap();
            segments += 1;
        }
    }

    fn render_food(&mut self) {
        write!(
            self.output.output,
            "{goto}{bgColor}{food_char}{hide}{reset}",
            goto = cursor::Goto(self.food.x, self.food.y),
            bgColor = color::Bg(color::Green),
            food_char = 'o',
            hide = cursor::Hide,
            reset = color::Bg(color::Reset),
        )
        .unwrap();
    }

    fn play(&mut self) {
        let mut direction_key = Key::Right;
        write!(self.output.output, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        self.output.output.flush().unwrap();
        'mainloop: loop {
            // Get input for direction
            'iter: loop {
                let temp = self.input.input.next();
                match temp {
                    Some(Ok(key)) => direction_key = key,
                    _ => break 'iter,
                }
            }
            match direction_key {
                Key::Char('q') => break 'mainloop,
                Key::Up
                    if (self.direction == Direction::Left
                        || self.direction == Direction::Right) =>
                {
                    self.direction = Direction::Up;
                }
                Key::Down
                    if (self.direction == Direction::Left
                        || self.direction == Direction::Right) =>
                {
                    self.direction = Direction::Down;
                }
                Key::Left
                    if (self.direction == Direction::Up || self.direction == Direction::Down) =>
                {
                    self.direction = Direction::Left;
                }
                Key::Right
                    if (self.direction == Direction::Up || self.direction == Direction::Down) =>
                {
                    self.direction = Direction::Right;
                }
                _ => (),
            }
            self.move_snake();
            if self.check_collision() {
                break 'mainloop;
            }
            if self.snake.front().unwrap() == &self.food {
                self.snake.push_back(self.snake.back().unwrap().clone());
                let grid_list = self.grid.iter().cloned().collect();
                self.generate_random_food(grid_list);
            }

            // Clear screen
            write!(self.output.output, "{}", clear::All).unwrap();
            self.render_snake();
            self.render_food();
            self.output.output.flush().unwrap();
            thread::sleep(Duration::from_millis(60));
        }

        // Reset terminal
        write!(
            self.output.output,
            "{}{}{}",
            termion::cursor::Show,
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = async_stdin().keys();
    let term_size = terminal_size().unwrap();
    let output = stdout().into_raw_mode().unwrap();

    let mut game = Game {
        grid: HashSet::new(),
        snake: VecDeque::new(),
        food: GridCell { x: 0, y: 0 },
        direction: Direction::Left,
        input: UserInput { input: input },
        output: Output { output: output },
        min_width: 1,
        min_height: 1,
        max_width: term_size.0,
        max_height: term_size.1,
    };
    game.initialize();
    game.play();

    Ok(())
}
