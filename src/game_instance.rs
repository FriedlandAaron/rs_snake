use std::collections::VecDeque;

use rand::Rng;

use crate::game::TerminalSize;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GridCell {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn vertical(&self) -> bool {
        match self {
            Self::Up | Self::Down => true,
            Self::Left | Self::Right => false,
        }
    }
}

const INIT_SNAKE_SIZE: u16 = 5;
#[derive(Debug)]
pub struct Snake {
    pub body: VecDeque<GridCell>,
    pub old_tail: Option<GridCell>,
}

impl Snake {
    fn new(grid: &GameGrid) -> Self {
        let mut body = VecDeque::new();
        let (_, y_min, x_max, y_max) = grid.get_corners();
        for i in 1..=INIT_SNAKE_SIZE {
            body.push_front(GridCell {
                x: x_max - i,
                y: (y_max + y_min) / 2,
            });
        }
        let old_tail = None;
        Self { body, old_tail }
    }
    fn get_head(&self) -> &GridCell {
        self.body.front().unwrap()
    }
    fn add_head(&mut self, head: GridCell) {
        self.body.push_front(head);
    }
    fn remove_tail(&mut self) {
        self.old_tail = self.body.pop_back();
    }
    fn restore_tail(&mut self) {
        self.body.push_back(self.old_tail.unwrap());
        self.old_tail = None;
    }
}

const TERM_MIN_COORD: f64 = 2.0;
#[derive(Debug)]
pub struct GameGrid {
    pub x_min: u16,
    pub y_min: u16,
    pub x_max: u16,
    pub y_max: u16,
    pub cells: Vec<GridCell>,
}

impl GameGrid {
    fn new(terminal_size: &TerminalSize, percent: f64) -> Self {
        let x_min = (TERM_MIN_COORD + ((terminal_size.x() - 1) as f64 * (1.0 - percent))) as u16;
        let y_min = (TERM_MIN_COORD + ((terminal_size.y() - 1) as f64 * (1.0 - percent))) as u16;
        let x_max = ((terminal_size.x() - 1) as f64 * percent) as u16;
        let y_max = ((terminal_size.y() - 1) as f64 * percent) as u16;
        let cells = Self::fill_cells(x_min, y_min, x_max, y_max);
        Self {
            x_min,
            y_min,
            x_max,
            y_max,
            cells,
        }
    }

    fn fill_cells(x_min: u16, y_min: u16, x_max: u16, y_max: u16) -> Vec<GridCell> {
        let mut cells = Vec::new();
        for i in x_min..=x_max {
            for j in y_min..=y_max {
                cells.push(GridCell { x: i, y: j });
            }
        }
        cells
    }

    pub fn get_corners(&self) -> (u16, u16, u16, u16) {
        (self.x_min, self.y_min, self.x_max, self.y_max)
    }
}

pub struct GameInstance {
    pub grid: GameGrid,
    pub snake: Snake,
    pub food: GridCell,
    pub direction: Direction,
}

impl GameInstance {
    pub fn new(terminal_size: &TerminalSize, grid_size: f64) -> Self {
        // Initialize grid
        let grid = GameGrid::new(terminal_size, grid_size);
        // Initialize snake
        let snake = Snake::new(&grid);
        // Generate food in a random cell
        let food = Self::generate_random_food(&grid.cells, &snake);
        // Initialize starting movement direction
        let direction = Direction::Left;
        Self {
            grid,
            snake,
            food,
            direction,
        }
    }

    pub fn new_welcome(terminal_size: &TerminalSize) -> Self {
        // Initialize grid
        let grid = GameGrid::new(terminal_size, 1.0);
        // Initialize snake
        let mut snake = Snake::new(&grid);
        let (_, _, x_max, y_max) = grid.get_corners();
        snake.body.clear();
        for i in 1..=INIT_SNAKE_SIZE {
            snake.body.push_front(GridCell {
                x: x_max - i,
                y: (y_max + (y_max / 2)) / 2,
            });
        }
        snake.old_tail = None;
        // Generate food in a random cell
        let food = Self::generate_random_food(&grid.cells, &snake);
        // Initialize starting movement direction
        let direction = Direction::Left;
        Self {
            grid,
            snake,
            food,
            direction,
        }
    }

    // return false if game over, else true
    pub fn game_cycle(&mut self) -> bool {
        // Handle snake movement
        self.move_snake();

        // Handle snake colliding with itself
        if self.check_collision() {
            return false;
        }
        // Handle snake eating food
        if self.snake.get_head() == &self.food {
            // Add another segment to the snake by restoring his old tail segment
            self.snake.restore_tail();
            // Generate new food
            self.food = GameInstance::generate_random_food(&self.grid.cells, &self.snake)
        }
        true
    }

    fn generate_random_food(cells: &[GridCell], snake: &Snake) -> GridCell {
        let mut rng = rand::thread_rng();
        let empty_cells: Vec<GridCell> = cells
            .iter()
            .cloned()
            .filter(|cell| !snake.body.contains(cell))
            .collect();
        let random_index = rng.gen_range(0..empty_cells.len());
        empty_cells[random_index]
    }

    fn move_snake(&mut self) {
        // Get current head
        let head = self.snake.get_head();

        // Create new head based on direction
        let new_head = match self.direction {
            // If snake is at an edge, wrap around to other side
            Direction::Right => {
                let x = if head.x == self.grid.x_max {
                    self.grid.x_min
                } else {
                    head.x + 1
                };
                let y = head.y;
                GridCell { x, y }
            }
            Direction::Left => {
                let x = if head.x == self.grid.x_min {
                    self.grid.x_max
                } else {
                    head.x - 1
                };
                let y = head.y;
                GridCell { x, y }
            }
            Direction::Up => {
                let x = head.x;
                let y = if head.y == self.grid.y_min {
                    self.grid.y_max
                } else {
                    head.y - 1
                };
                GridCell { x, y }
            }
            Direction::Down => {
                let x = head.x;
                let y = if head.y == self.grid.y_max {
                    self.grid.y_min
                } else {
                    head.y + 1
                };
                GridCell { x, y }
            }
        };

        // Push new head to start of snake
        self.snake.add_head(new_head);

        // Return old tail from snake
        self.snake.remove_tail()
    }

    fn check_collision(&mut self) -> bool {
        let mut collision = false;
        let head = self.snake.get_head();

        for segment in self.snake.body.range(1..) {
            if head == segment {
                collision = true;
                break;
            }
        }
        collision
    }
}
