use std::{thread, time::Duration};

use crate::game_input::{self, KeyPress};
use crate::game_instance::{Direction, GameInstance};
use crate::game_output;
use crate::parser::{ArgsParser, GridSize, MovementKeyScheme, Speed};

struct Options {
    grid_size: GridSize,
    speed: Speed,
    movement_key_scheme: MovementKeyScheme,
}

impl Options {
    fn new(grid_size: GridSize, speed: Speed, movement_key_scheme: MovementKeyScheme) -> Self {
        Self {
            grid_size,
            speed,
            movement_key_scheme,
        }
    }
    fn from_args(args: ArgsParser) -> Self {
        Options::new(args.grid_size, args.speed, args.movement_key_scheme)
    }
    fn new_pre_game() -> Self {
        Options::new(GridSize::Large, Speed::High, MovementKeyScheme::Arrows)
    }
}

pub struct TerminalSize {
    xy: (u16, u16),
}

impl TerminalSize {
    fn new(xy: (u16, u16)) -> Self {
        Self { xy }
    }
    pub fn x(&self) -> u16 {
        self.xy.0
    }
    pub fn y(&self) -> u16 {
        self.xy.1
    }
}

#[derive(Debug, PartialEq)]
enum GameState {
    PreGame,
    InProgress,
    GameOver,
    QuitButtonPressed,
    RestartGame,
    GameOverTransition,
}

pub struct Game {
    options: Options,
    state: GameState,
    instance: GameInstance,
    input: game_input::GameInput,
    output: game_output::GameOutput,
    terminal_size: TerminalSize,
}

impl Game {
    pub fn new(
        args: ArgsParser,
        input: game_input::GameInput,
        output: game_output::GameOutput,
        terminal_size: (u16, u16),
    ) -> Game {
        let terminal_size = TerminalSize::new(terminal_size);
        let state = GameState::PreGame;
        let options = Options::from_args(args);
        let instance = GameInstance::new(&terminal_size, options.grid_size.value());
        Game {
            options,
            state,
            instance,
            input,
            output,
            terminal_size,
        }
    }

    // return value = new state? or middleman function interprets return value and gives new state?
    pub fn run(&mut self) {
        loop {
            match self.state {
                GameState::PreGame => {
                    self.state = self.pre_game();
                }
                GameState::InProgress => {
                    self.state = self.in_progress_game();
                }
                GameState::GameOverTransition => {
                    self.state = self.game_over_transition();
                }
                GameState::GameOver => {
                    self.state = self.game_over();
                }
                GameState::RestartGame => {
                    self.state = self.restart_game();
                }
                GameState::QuitButtonPressed => {
                    break;
                }
            }
        }
        self.output.show_cursor();
    }

    fn pre_game(&mut self) -> GameState {
        self.instance = GameInstance::new_pre_game(&self.terminal_size);
        self.output.clear_screen();
        self.output.draw_pre_game_message();
        self.output.draw_snake(&self.instance.snake);
        self.output.render();
        loop {
            match self.input.get_keypress() {
                // Start playing the game
                KeyPress::Pause => break,
                // Quit the game
                KeyPress::Quit => return GameState::QuitButtonPressed,
                _ => (),
            }
            self.instance.game_cycle();
            self.output.draw_snake(&self.instance.snake);
            self.output.render();

            // Sleep here to let input thread have some control
            thread::sleep(Duration::from_millis(self.options.speed.value()));
        }
        GameState::InProgress
    }

    fn in_progress_game(&mut self) -> GameState {
        self.instance = GameInstance::new(&self.terminal_size, self.options.grid_size.value());
        // Initial render
        self.output.clear_screen();
        self.draw_border();
        self.output.draw_snake(&self.instance.snake);
        self.output.draw_food(&self.instance.food);
        self.output.render();

        // Start of main loop
        'mainloop: loop {
            // Handle user input
            match self.input.get_keypress() {
                // Pause the game
                KeyPress::Pause => {
                    while let KeyPress::None | KeyPress::Other = self.input.get_keypress() {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                // Quit the game
                KeyPress::Quit => return GameState::QuitButtonPressed,
                // Get pressed direction key
                KeyPress::DirectionKey(Direction::Up) if !self.instance.direction.vertical() => {
                    self.instance.direction = Direction::Up;
                }
                KeyPress::DirectionKey(Direction::Down) if !self.instance.direction.vertical() => {
                    self.instance.direction = Direction::Down;
                }
                KeyPress::DirectionKey(Direction::Left) if self.instance.direction.vertical() => {
                    self.instance.direction = Direction::Left;
                }
                KeyPress::DirectionKey(Direction::Right) if self.instance.direction.vertical() => {
                    self.instance.direction = Direction::Right;
                }
                _ => (),
            }

            let proceed = self.instance.game_cycle();

            if !proceed {
                break 'mainloop;
            }
            self.output.draw_snake(&self.instance.snake);
            self.output.draw_food(&self.instance.food);
            self.output.render();
            thread::sleep(Duration::from_millis(
                if self.instance.direction.vertical() {
                    self.options.speed.value() + 20
                } else {
                    self.options.speed.value()
                },
            ));
        }
        GameState::GameOverTransition
    }

    fn game_over(&mut self) -> GameState {
        // Clear terminal
        self.output.clear_screen();

        // Render game over screen
        self.output
            .draw_game_over_message(self.instance.snake.body.len());
        self.output.render();

        // Handle input
        self.input.empty_key_buffer();
        loop {
            match self.input.get_keypress() {
                KeyPress::Pause => return GameState::RestartGame,
                KeyPress::Quit => return GameState::QuitButtonPressed,
                _ => (),
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn restart_game(&mut self) -> GameState {
        self.instance = GameInstance::new(&self.terminal_size, self.options.grid_size.value());
        GameState::InProgress
    }

    fn game_over_transition(&mut self) -> GameState {
        let transition_time = 500;
        let num_changes = 3;
        for _ in 1..=num_changes {
            self.output
                .draw_game_over_transition_msg(self.instance.grid.y_min, self.instance.grid.y_max);
            self.draw_border();
            self.output.render();
            thread::sleep(Duration::from_millis(transition_time));
            self.output.clear_screen();
            self.draw_all();
            self.output.render();
            thread::sleep(Duration::from_millis(transition_time));
        }
        GameState::GameOver
    }

    fn draw_border(&mut self) {
        let (x_min, y_min, x_max, y_max) = self.instance.grid.get_corners();
        self.output.draw_border(x_min, x_max, y_min, y_max);
    }

    fn draw_all(&mut self) {
        self.draw_border();
        self.output.draw_snake(&self.instance.snake);
        self.output.draw_food(&self.instance.food);
    }
}
