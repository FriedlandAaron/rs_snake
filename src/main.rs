use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::io::{stdout, Stdout, Write};
use std::{thread, time::Duration};

use rand::Rng;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, color, cursor, terminal_size, AsyncReader};

#[derive(Debug)]
enum KeyPress {
    Direction(Direction),
    Q,
    None,
}

#[derive(Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// TODO: still need to figure out how to abstract this part properly
struct UserInput {
    input: termion::input::Keys<AsyncReader>,
}

impl UserInput {
    fn get_keypress(&mut self) -> KeyPress {
        match self.input.by_ref().last() {
            Some(Ok(key)) => match key {
                Key::Char('q') => KeyPress::Q,
                Key::Up => KeyPress::Direction(Direction::Up),
                Key::Down => KeyPress::Direction(Direction::Down),
                Key::Left => KeyPress::Direction(Direction::Left),
                Key::Right => KeyPress::Direction(Direction::Right),
                _ => KeyPress::None,
            },
            _ => KeyPress::None,
        }
    }
}

// TODO: still need to figure out how to abstract this part properly
struct Output {
    output: termion::raw::RawTerminal<Stdout>,
}

impl Output {
    fn render(&mut self) {
        self.output.flush().unwrap();
    }

    fn clear_screen(&mut self) {
        write!(self.output, "{}", clear::All).unwrap();
    }

    fn reset_terminal(&mut self) {
        write!(
            self.output,
            "{}{}{}",
            termion::cursor::Show,
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();
    }

    fn draw_border(&mut self, xmin: u16, xmax: u16, ymin: u16, ymax: u16) {
        for i in xmin - 1..=xmax + 1 {
            for j in ymin - 1..=ymax + 1 {
                match i {
                    n if (n == xmin - 1 || n == xmax + 1) => write!(
                        self.output,
                        "{goto}{bgColor}.",
                        goto = cursor::Goto(i, j),
                        bgColor = color::Bg(color::White),
                    )
                    .unwrap(),
                    _ => (),
                };
                match j {
                    n if (n == ymin - 1 || n == ymax + 1) => write!(
                        self.output,
                        "{goto}{bgColor}.",
                        goto = cursor::Goto(i, j),
                        bgColor = color::Bg(color::White),
                    )
                    .unwrap(),
                    _ => (),
                };
            }
        }
        write!(self.output, "{}", color::Bg(color::Reset),).unwrap()
    }

    fn draw_food(&mut self, food: &GridCell) {
        write!(
            self.output,
            "{goto}{bgColor}{food_char}{hide}{reset}",
            goto = cursor::Goto(food.x, food.y),
            bgColor = color::Bg(color::Green),
            food_char = 'o',
            hide = cursor::Hide,
            reset = color::Bg(color::Reset),
        )
        .unwrap();
    }

    fn draw_snake(&mut self, snake: &VecDeque<GridCell>) {
        let mut segments: usize = 0;
        let len = snake.len();
        for segment in snake {
            let segment_char = match segments {
                0 => 'S',
                num if num == len - 1 => 'e',
                1 => 'n',
                num if num == len - 2 => 'k',
                _ => 'a',
            };
            write!(
                self.output,
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
        for i in self.min_width..=self.max_width {
            for j in self.min_height..=self.max_height {
                self.grid.insert(GridCell { x: i, y: j });
            }
        }

        // Initialize snake
        let init_size = 5;
        for i in 1..=init_size {
            self.snake.push_front(GridCell {
                x: self.max_width - i,
                y: (self.max_height + self.min_height) / 2,
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
            // If snake is at an edge, wrap around to other side
            Direction::Right if head.x == self.max_width => GridCell {
                x: self.min_width,
                y: head.y,
            },
            Direction::Left if head.x == self.min_width => GridCell {
                x: self.max_width,
                y: head.y,
            },
            Direction::Up if head.y == self.min_height => GridCell {
                x: head.x,
                y: self.max_height,
            },
            Direction::Down if head.y == self.max_height => GridCell {
                x: head.x,
                y: self.min_height,
            },
            // If snake isn't at an edge, advance one cell in the chosen direction
            Direction::Right => GridCell {
                x: head.x + 1,
                y: head.y,
            },
            Direction::Left => GridCell {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Up => GridCell {
                x: head.x,
                y: head.y - 1,
            },
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

    fn play(&mut self) {
        // Initial render to clear screen
        self.output.clear_screen();
        self.output.render();

        // Start of main loop
        'mainloop: loop {
            // Get input for direction
            match self.input.get_keypress() {
                KeyPress::Q => break 'mainloop,
                KeyPress::Direction(Direction::Up)
                    if (self.direction == Direction::Left
                        || self.direction == Direction::Right) =>
                {
                    self.direction = Direction::Up;
                }
                KeyPress::Direction(Direction::Down)
                    if (self.direction == Direction::Left
                        || self.direction == Direction::Right) =>
                {
                    self.direction = Direction::Down;
                }
                KeyPress::Direction(Direction::Left)
                    if (self.direction == Direction::Up || self.direction == Direction::Down) =>
                {
                    self.direction = Direction::Left;
                }
                KeyPress::Direction(Direction::Right)
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
            thread::sleep(Duration::from_millis(60));
        }

        // Reset terminal
        self.output.reset_terminal();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = async_stdin().keys();
    let output = stdout().into_raw_mode().unwrap();

    let term_size = terminal_size().unwrap();
    let playable = 0.7;

    let mut game = Game {
        grid: HashSet::new(),
        snake: VecDeque::new(),
        food: GridCell { x: 0, y: 0 },
        direction: Direction::Left,
        input: UserInput { input: input },
        output: Output { output: output },
        min_width: (2.0 + ((term_size.0 - 1) as f64 * (1.0 - playable))) as u16,
        min_height: (2.0 + ((term_size.1 - 1) as f64 * (1.0 - playable))) as u16,
        max_width: ((term_size.0 - 1) as f64 * playable) as u16,
        max_height: ((term_size.1 - 1) as f64 * playable) as u16,
    };

    game.initialize();
    game.play();

    Ok(())
}
