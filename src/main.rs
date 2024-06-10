use std::error::Error;
use std::io::stdout;

use clap::Parser;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use termion::{async_stdin, terminal_size};

mod game;
mod game_input;
mod game_instance;
mod game_output;
mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse arguments from command line
    let args = parser::ArgsParser::parse();
    // Initialize input handler
    let input = async_stdin().keys();
    let input = game_input::GameInput::new(input, args.movement_key_scheme);
    // Initialize output handler
    let output = stdout().into_raw_mode()?.into_alternate_screen()?;
    let output = game_output::GameOutput::new(output);

    // Initialize rest of variables needed to initialize Game struct
    let terminal_size = terminal_size()?;

    let mut game = game::Game::new(args, input, output, terminal_size);

    game.run();

    Ok(())
}
