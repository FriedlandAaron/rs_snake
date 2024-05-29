use rand::Rng;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
use std::{thread, time::Duration};

use crate::game_input::{self, KeyPress};
use crate::game_output;

const INIT_SNAKE_SIZE: u16 = 5;

#[derive(Debug, PartialEq)]
enum GameState {
    PreGame,
    InProgress,
    GameOver,
}

#[derive(Debug, PartialEq)]
pub enum Direction {
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GridCell {
    pub x: u16,
    pub y: u16,
}

pub struct Game {
    state: GameState,
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
    pub fn new(
        input: game_input::GameInput,
        output: game_output::GameOutput,
        term_max_x: u16,
        term_max_y: u16,
        playable: f64,
        speed: u64,
    ) -> Game {
        let state = GameState::PreGame;
        let min_width = (2.0 + ((term_max_x - 1) as f64 * (1.0 - playable))) as u16;
        let min_height = (2.0 + ((term_max_y - 1) as f64 * (1.0 - playable))) as u16;
        let max_width = ((term_max_x - 1) as f64 * playable) as u16;
        let max_height = ((term_max_y - 1) as f64 * playable) as u16;

        // Initialize game grid
        let mut grid = Self::create_grid(min_width, min_height, max_width, max_height);

        // Initialize snake
        let snake = Self::create_snake(min_height, max_width, max_height);

        // Update grid by removing cells occupied by snake
        for seg in &snake {
            grid.remove(seg);
        }

        // Generate food in a random cell
        let food = Game::generate_random_food(&grid);
        grid.remove(&food);

        // Initialize starting movement direction
        let direction = Direction::Left;

        Game {
            state,
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

    pub fn run(&mut self) {
        loop {
            match self.state {
                // TODO: handle pregame
                GameState::PreGame => self.state = GameState::InProgress,
                GameState::InProgress => {
                    let quit = self.play();
                    if quit {
                        break;
                    }
                }
                GameState::GameOver => {
                    let keep_playing = self.game_over();
                    if keep_playing {
                        self.restart();
                    } else {
                        break;
                    }
                }
            }
        }

        // Reset terminal
        self.output.reset_terminal();
    }

    fn create_grid(min_w: u16, min_h: u16, max_w: u16, max_h: u16) -> HashSet<GridCell> {
        let mut grid = HashSet::new();
        for i in min_w..=max_w {
            for j in min_h..=max_h {
                grid.insert(GridCell { x: i, y: j });
            }
        }
        grid
    }

    fn create_snake(min_h: u16, max_w: u16, max_h: u16) -> VecDeque<GridCell> {
        let mut snake = VecDeque::new();
        for i in 1..=INIT_SNAKE_SIZE {
            snake.push_front(GridCell {
                x: max_w - i,
                y: (max_h + min_h) / 2,
            });
        }
        snake
    }

    fn restart(&mut self) {
        self.state = GameState::InProgress;

        // Initialize game grid
        self.grid = Self::create_grid(
            self.min_width,
            self.min_height,
            self.max_width,
            self.max_height,
        );

        // Initialize snake
        self.snake = Self::create_snake(self.min_height, self.max_width, self.max_height);

        // Update grid by removing cells occupied by snake
        for seg in &self.snake {
            self.grid.remove(seg);
        }

        // Generate food in a random cell
        self.food = Game::generate_random_food(&self.grid);
        self.grid.remove(&self.food);

        // Initialize starting movement direction
        self.direction = Direction::Left;
    }

    fn generate_random_food(grid: &HashSet<GridCell>) -> GridCell {
        let mut rng = rand::thread_rng();
        let grid_list: Vec<&GridCell> = grid.iter().by_ref().collect();
        let random_index = rng.gen_range(0..grid_list.len());
        grid_list[random_index].clone()
    }

    fn move_snake(&mut self) -> GridCell {
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

        // Return old tail from snake
        self.snake.pop_back().unwrap()
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

    fn game_over(&mut self) -> bool {
        let mut keep_playing = false;
        // Clear terminal
        self.output.clear_screen();

        // Render game over screen
        self.output.draw_game_over_message(self.snake.len());
        self.output.render();

        // Handle input
        loop {
            match self.input.get_keypress() {
                KeyPress::Pause => {
                    keep_playing = true;
                    break;
                }
                KeyPress::Quit => break,
                _ => (),
            }
            thread::sleep(Duration::from_millis(10));
        }
        keep_playing
    }

    fn play(&mut self) -> bool {
        // Initial render to clear screen and draw borders and food
        self.output.clear_screen();
        self.output.draw_border(
            self.min_width,
            self.max_width,
            self.min_height,
            self.max_height,
        );
        self.output.draw_food(&self.food);
        self.output.render();
        // Start of main loop
        'mainloop: loop {
            // Handle user input
            match self.input.get_keypress() {
                // Pause the game
                KeyPress::Pause => loop {
                    match self.input.get_keypress() {
                        KeyPress::None | KeyPress::Other => (),
                        _ => break,
                    }
                    // Sleep here to let input thread have some control
                    thread::sleep(Duration::from_millis(10));
                },
                // Quit the game
                KeyPress::Quit => return true,
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
            // Handle snake movement
            let old_tail = self.move_snake();

            // Handle snake colliding with itself
            if self.check_collision() {
                self.state = GameState::GameOver;
                break 'mainloop;
            }
            // Handle snake eating food
            if self.snake.front().unwrap() == &self.food {
                for seg in &self.snake {
                    self.grid.remove(seg);
                }
                // Add another segment to the snake by restoring his old tail segment
                self.snake.push_back(old_tail);
                // Undraw old food and restore cell to grid
                self.grid.insert(self.food);
                // Generate new food, draw it and remove cell from grid
                self.food = Game::generate_random_food(&self.grid);
                self.output.draw_food(&self.food);
                self.grid.remove(&self.food);
            } else {
                self.output.undraw(&old_tail);
                self.grid.insert(old_tail);
            }
            self.output.draw_snake(&self.snake);
            self.output.render();
            thread::sleep(Duration::from_millis(if self.direction.vertical() {
                self.speed + 20
            } else {
                self.speed
            }));
        }
        false
    }
}
