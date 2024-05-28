use std::collections::HashMap;

use termion::event::Key;
use termion::AsyncReader;

use crate::Direction;

#[derive(Debug, PartialEq)]
pub enum KeyPress {
    ArrowKey(Direction),
    Q,
    P,
    Other,
    None,
}

// TODO: still need to figure out how to abstract this part properly
pub struct GameInput {
    pub input: termion::input::Keys<AsyncReader>,
    pub keybinds: HashMap<termion::event::Key, KeyPress>,
}

impl GameInput {
    pub fn new(input: termion::input::Keys<AsyncReader>) -> Self {
        Self {
            input,
            keybinds: Self::create_keybinds(),
        }
    }
    fn create_keybinds() -> HashMap<termion::event::Key, KeyPress> {
        let mut keybinds = HashMap::new();

        // Insert Quit button
        keybinds.insert(Key::Char('q'), KeyPress::Q);
        keybinds.insert(Key::Char('Q'), KeyPress::Q);
        // Insert Pause button
        keybinds.insert(Key::Char('p'), KeyPress::P);
        keybinds.insert(Key::Char('P'), KeyPress::P);
        // Insert direction buttons
        keybinds.insert(Key::Up, KeyPress::ArrowKey(Direction::Up));
        keybinds.insert(Key::Down, KeyPress::ArrowKey(Direction::Down));
        keybinds.insert(Key::Left, KeyPress::ArrowKey(Direction::Left));
        keybinds.insert(Key::Right, KeyPress::ArrowKey(Direction::Right));

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
