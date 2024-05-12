use std::collections::{HashSet, VecDeque};
use std::error::Error;
use std::io::{stdout, Stdout, Write};
use std::{thread, time::Duration};

use rand::Rng;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{async_stdin, clear, color, cursor, terminal_size};

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct GridCell {
    x: u16,
    y: u16,
}

struct Snake {
    body: VecDeque<GridCell>,
}

impl Snake {
    fn move_snake(&mut self, direction: &Direction, term_size: (u16, u16)) {
        // Get current head
        let head = self.body.front().unwrap();

        // Create new head based on direction
        let new_head = match direction {
            Direction::Right if head.x == term_size.0 => GridCell { x: 1, y: head.y },
            Direction::Right => GridCell {
                x: head.x + 1,
                y: head.y,
            },
            Direction::Left if head.x == 1 => GridCell {
                x: term_size.0,
                y: head.y,
            },
            Direction::Left => GridCell {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Up if head.y == 0 => GridCell {
                x: head.x,
                y: term_size.1,
            },
            Direction::Up => GridCell {
                x: head.x,
                y: head.y - 1,
            },
            Direction::Down if head.y == term_size.1 => GridCell { x: head.x, y: 1 },
            Direction::Down => GridCell {
                x: head.x,
                y: head.y + 1,
            },
        };

        // Push new head to start of snake
        self.body.push_front(new_head);

        // Remove old tail from snake
        self.body.pop_back();
    }

    fn check_collision(&mut self) -> bool {
        let mut collision = false;
        let head = self.body.front().unwrap();

        for segment in self.body.range(1..) {
            if head == segment {
                collision = true;
                break;
            }
        }
        collision
    }
}

fn render_snake(screen: &mut RawTerminal<Stdout>, snake: &Snake) {
    let mut segments: usize = 0;
    let len = snake.body.len();
    for segment in &snake.body {
        let segment_char = match segments {
            0 => 'H',
            num if num == len - 1 => 'T',
            _ => 'm',
        };
        write!(
            screen,
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

fn render_food(screen: &mut RawTerminal<Stdout>, food: &GridCell) {
    write!(
        screen,
        "{goto}{bgColor}{food_char}{hide}{reset}",
        goto = cursor::Goto(food.x, food.y),
        bgColor = color::Bg(color::Green),
        food_char = 'F',
        hide = cursor::Hide,
        reset = color::Bg(color::Reset),
    )
    .unwrap();
}

fn generate_random_food(grid: &HashSet<GridCell>) -> &GridCell {
    let mut rng = rand::thread_rng();
    let grid_list: Vec<_> = grid.iter().collect();
    let random_index = rng.gen_range(0..grid_list.len());
    let cell = grid_list[random_index];
    cell
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdin = async_stdin().keys();
    let term_size = terminal_size().unwrap();

    let mut grid: HashSet<GridCell> = HashSet::new();
    for i in 1..term_size.0 {
        for j in 1..term_size.1 {
            grid.insert(GridCell { x: i, y: j });
        }
    }

    let mut snake2 = Snake {
        body: VecDeque::new(),
    };
    snake2.body.push_back(GridCell {
        x: term_size.0 - 1,
        y: term_size.1 / 2,
    });
    snake2.body.push_back(GridCell {
        x: term_size.0 - 2,
        y: term_size.1 / 2,
    });
    snake2.body.push_back(GridCell {
        x: term_size.0 - 3,
        y: term_size.1 / 2,
    });

    for seg in &snake2.body {
        grid.remove(&seg);
    }

    let mut food = generate_random_food(&grid);

    let mut direction = Direction::Right;
    let mut direction_key = Key::Right;
    let mut screen = stdout().into_raw_mode().unwrap();
    write!(screen, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    screen.flush().unwrap();
    'mainloop: loop {
        // Clear screen
        write!(screen, "{}", clear::All).unwrap();

        // Get input for direction
        'iter: loop {
            let temp = stdin.next();
            match temp {
                Some(Ok(key)) => direction_key = key,
                _ => break 'iter,
            }
        }
        match direction_key {
            Key::Char('q') => break 'mainloop,
            Key::Up if (direction == Direction::Left || direction == Direction::Right) => {
                direction = Direction::Up;
            }
            Key::Down if (direction == Direction::Left || direction == Direction::Right) => {
                direction = Direction::Down;
            }
            Key::Left if (direction == Direction::Up || direction == Direction::Down) => {
                direction = Direction::Left;
            }
            Key::Right if (direction == Direction::Up || direction == Direction::Down) => {
                direction = Direction::Right;
            }
            _ => (),
        }
        snake2.move_snake(&direction, term_size);
        if snake2.body.front().unwrap() == food {
            snake2.body.push_back(snake2.body.back().unwrap().clone());
            food = generate_random_food(&grid);
        }
        render_snake(&mut screen, &snake2);
        render_food(&mut screen, food);
        screen.flush().unwrap();
        if snake2.check_collision() {
            break 'mainloop;
        }
        thread::sleep(Duration::from_millis(80));
    }

    // Reset terminal
    write!(
        screen,
        "{}{}{}",
        termion::cursor::Show,
        termion::cursor::Goto(1, 1),
        termion::clear::All
    )
    .unwrap();

    Ok(())
}
