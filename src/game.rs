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
        let instance = GameInstance::new(&terminal_size, args.grid_size.value());
        let options = Options {
            grid_size: args.grid_size,
            speed: args.speed,
            movement_key_scheme: args.movement_key_scheme,
        };
        Game {
            options,
            state,
            instance,
            input,
            output,
            terminal_size,
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
                    self.game_over_transition();
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

    fn play(&mut self) -> bool {
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
                self.state = GameState::GameOver;
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
        false
    }

    fn game_over(&mut self) -> bool {
        let mut keep_playing = false;
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

    fn restart(&mut self) {
        self.state = GameState::InProgress;
        self.instance = GameInstance::new(&self.terminal_size, self.options.grid_size.value());
    }

    fn game_over_transition(&mut self) {
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
