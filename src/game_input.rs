use termion::event::Key;
use termion::AsyncReader;

use crate::Direction;

#[derive(Debug, PartialEq)]
pub enum KeyPress {
    Direction(Direction),
    Q,
    P,
    Other,
    None,
}

// TODO: still need to figure out how to abstract this part properly
pub struct GameInput {
    pub input: termion::input::Keys<AsyncReader>,
}

impl GameInput {
    pub fn get_keypress(&mut self) -> KeyPress {
        match self.input.by_ref().last() {
            Some(Ok(key)) => match key {
                Key::Char('q') => KeyPress::Q,
                Key::Char('p') => KeyPress::P,
                Key::Up => KeyPress::Direction(Direction::Up),
                Key::Down => KeyPress::Direction(Direction::Down),
                Key::Left => KeyPress::Direction(Direction::Left),
                Key::Right => KeyPress::Direction(Direction::Right),
                _ => KeyPress::Other,
            },
            _ => KeyPress::None,
        }
    }
}
