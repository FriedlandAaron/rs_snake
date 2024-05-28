use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::io::stdout;
use std::{thread, time::Duration};

use clap::Parser;
use rand::Rng;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use termion::{async_stdin, terminal_size};

use game_input::KeyPress;
mod game_input;
mod game_output;
mod parser;

#[derive(Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn vertical(&self) -> bool {
        match self {
            Self::Up | Self::Down => true,
            Self::Left | Self::Right => false,
        }
    }
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
    input: game_input::GameInput,
    output: game_output::GameOutput,
    min_width: u16,
    min_height: u16,
    max_width: u16,
    max_height: u16,
    speed: u64,
}

impl Game {
    fn new(
        input: game_input::GameInput,
        output: game_output::GameOutput,
        term_max_x: u16,
        term_max_y: u16,
        playable: f64,
        speed: u64,
    ) -> Game {
        let min_width = (2.0 + ((term_max_x - 1) as f64 * (1.0 - playable))) as u16;
        let min_height = (2.0 + ((term_max_y - 1) as f64 * (1.0 - playable))) as u16;
        let max_width = ((term_max_x - 1) as f64 * playable) as u16;
        let max_height = ((term_max_y - 1) as f64 * playable) as u16;

        // Initialize game grid
        let mut grid = HashSet::new();
        for i in min_width..=max_width {
            for j in min_height..=max_height {
                grid.insert(GridCell { x: i, y: j });
            }
        }

        // Initialize snake
        let mut snake = VecDeque::new();
        let init_size = 5;
        for i in 1..=init_size {
            snake.push_front(GridCell {
                x: max_width - i,
                y: (max_height + min_height) / 2,
            });
        }

        // Update grid by removing cells occupied by snake
        for seg in &snake {
            grid.remove(seg);
        }

        // Generate food in a random cell
        let food = Game::generate_random_food(&grid);

        // Initialize starting movement direction
        let direction = Direction::Left;

        Game {
            grid,
            snake,
            food,
            direction,
            input,
            output,
            min_width,
            min_height,
            max_width,
            max_height,
            speed,
        }
    }

    fn generate_random_food(grid: &HashSet<GridCell>) -> GridCell {
        let mut rng = rand::thread_rng();
        let grid_list: Vec<&GridCell> = grid.iter().by_ref().collect();
        let random_index = rng.gen_range(0..grid_list.len());
        grid_list[random_index].clone()
    }

    fn move_snake(&mut self) {
        // Get current head
        let head = self.snake.front().unwrap();

        // Create new head based on direction
        let new_head = match self.direction {
            // If snake is at an edge, wrap around to other side
            Direction::Right => {
                let x = if head.x == self.max_width {
                    self.min_width
                } else {
                    head.x + 1
                };
                let y = head.y;
                GridCell { x, y }
            }
            Direction::Left => {
                let x = if head.x == self.min_width {
                    self.max_width
                } else {
                    head.x - 1
                };
                let y = head.y;
                GridCell { x, y }
            }
            Direction::Up => {
                let x = head.x;
                let y = if head.y == self.min_height {
                    self.max_height
                } else {
                    head.y - 1
                };
                GridCell { x, y }
            }
            Direction::Down => {
                let x = head.x;
                let y = if head.y == self.max_height {
                    self.min_height
                } else {
                    head.y + 1
                };
                GridCell { x, y }
            }
        };

        // Remove new head from grid
        self.grid.remove(&new_head);
        // Push new head to start of snake
        self.snake.push_front(new_head);

        // Remove old tail from snake
        let old_tail = self.snake.pop_back().unwrap();
        // Put old tail back into grid
        self.grid.insert(old_tail);
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

    fn play(&mut self) {
        // Initial render to clear screen
        self.output.clear_screen();
        self.output.render();

        // Start of main loop
        'mainloop: loop {
            // Handle user input
            match self.input.get_keypress() {
                // Pause the game
                KeyPress::Pause => loop {
                    // Sleep here to let input thread have some control
                    thread::sleep(Duration::from_millis(10));
                    match self.input.get_keypress() {
                        KeyPress::None | KeyPress::Other => (),
                        _ => break,
                    }
                },
                // Quit the game
                KeyPress::Quit => break 'mainloop,
                // Get pressed direction key
                KeyPress::DirectionKey(Direction::Up) if !self.direction.vertical() => {
                    self.direction = Direction::Up;
                }
                KeyPress::DirectionKey(Direction::Down) if !self.direction.vertical() => {
                    self.direction = Direction::Down;
                }
                KeyPress::DirectionKey(Direction::Left) if self.direction.vertical() => {
                    self.direction = Direction::Left;
                }
                KeyPress::DirectionKey(Direction::Right) if self.direction.vertical() => {
                    self.direction = Direction::Right;
                }
                _ => (),
            }
            self.move_snake();
            if self.check_collision() {
                break 'mainloop;
            }
            if self.snake.front().unwrap() == &self.food {
                for seg in &self.snake {
                    self.grid.remove(seg);
                }
                self.snake.push_back(self.snake.back().unwrap().clone());
                self.food = Game::generate_random_food(&self.grid);
                self.grid.remove(&self.food);
            }

            // Clear screen
            self.output.clear_screen();
            self.output.draw_border(
                self.min_width,
                self.max_width,
                self.min_height,
                self.max_height,
            );
            self.output.draw_snake(&self.snake);
            self.output.draw_food(&self.food);
            self.output.render();
            thread::sleep(Duration::from_millis(if self.direction.vertical() {
                self.speed + 20
            } else {
                self.speed
            }));
        }

        // Reset terminal
        self.output.reset_terminal();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parser::ArgsParser::parse();
    let input = async_stdin().keys();
    let input = game_input::GameInput::new(input, args.movement_key_scheme);
    let output = stdout().into_raw_mode()?.into_alternate_screen()?;

    let term_size = terminal_size()?;
    let playable = args.grid_size.value();
    let speed = args.speed.value();

    let mut game = Game::new(
        input,
        game_output::GameOutput { output },
        term_size.0,
        term_size.1,
        playable,
        speed,
    );

    game.play();

    Ok(())
}
