use std::collections::HashMap;

use termion::event::Key;
use termion::AsyncReader;

use crate::game::Direction;
use crate::parser::MovementKeyScheme;

#[derive(Debug, PartialEq)]
pub enum KeyPress {
    DirectionKey(Direction),
    Quit,
    Pause,
    Other,
    None,
}

// TODO: still need to figure out how to abstract this part properly
pub struct GameInput {
    pub input: termion::input::Keys<AsyncReader>,
    pub keybinds: HashMap<termion::event::Key, KeyPress>,
}

impl GameInput {
    pub fn new(input: termion::input::Keys<AsyncReader>, key_scheme: MovementKeyScheme) -> Self {
        Self {
            input,
            keybinds: Self::create_keybinds(key_scheme),
        }
    }
    fn create_keybinds(
        movement_key_scheme: MovementKeyScheme,
    ) -> HashMap<termion::event::Key, KeyPress> {
        let mut keybinds = HashMap::new();

        // Insert Quit button
        keybinds.insert(Key::Char('q'), KeyPress::Quit);
        keybinds.insert(Key::Char('Q'), KeyPress::Quit);
        // Insert Pause button
        keybinds.insert(Key::Char('p'), KeyPress::Pause);
        keybinds.insert(Key::Char('P'), KeyPress::Pause);
        // Insert direction buttons
        match movement_key_scheme {
            MovementKeyScheme::Arrows => {
                keybinds.insert(Key::Up, KeyPress::DirectionKey(Direction::Up));
                keybinds.insert(Key::Down, KeyPress::DirectionKey(Direction::Down));
                keybinds.insert(Key::Left, KeyPress::DirectionKey(Direction::Left));
                keybinds.insert(Key::Right, KeyPress::DirectionKey(Direction::Right));
            }
            MovementKeyScheme::WSAD => {
                keybinds.insert(Key::Char('w'), KeyPress::DirectionKey(Direction::Up));
                keybinds.insert(Key::Char('s'), KeyPress::DirectionKey(Direction::Down));
                keybinds.insert(Key::Char('a'), KeyPress::DirectionKey(Direction::Left));
                keybinds.insert(Key::Char('d'), KeyPress::DirectionKey(Direction::Right));
            }
        }
        keybinds
    }

    pub fn get_keypress(&mut self) -> &KeyPress {
        match self.input.by_ref().last() {
            Some(result) => {
                let key = result.unwrap();
                match self.keybinds.contains_key(&key) {
                    true => self.keybinds.get(&key).unwrap(),
                    false => &KeyPress::Other,
                }
            }
            None => &KeyPress::None,
        }
    }
}
