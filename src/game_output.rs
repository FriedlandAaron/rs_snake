use std::io::{Stdout, Write};

use cfonts::{Align, Colors, Fonts, Options};
use termion::raw::RawTerminal;
use termion::{clear, color, cursor};

use crate::game_instance::GridCell;
use crate::game_instance::Snake;

const FOOD_CHAR: char = '\u{00D3}';

// TODO: still need to figure out how to abstract this part properly
pub struct GameOutput {
    output: termion::screen::AlternateScreen<RawTerminal<Stdout>>,
}

impl GameOutput {
    pub fn new(output: termion::screen::AlternateScreen<RawTerminal<Stdout>>) -> Self {
        Self { output }
    }
    pub fn render(&mut self) {
        self.output.flush().unwrap();
    }

    pub fn clear_screen(&mut self) {
        write!(self.output, "{}{}", clear::All, cursor::Hide).unwrap();
    }

    pub fn draw_game_over_transition_msg(&mut self, min_y: u16, max_y: u16) {
        let msg = cfonts::render(Options {
            text: String::from("game|over!"),
            font: Fonts::FontBlock,
            align: Align::Center,
            line_height: 0,
            colors: vec![Colors::RedBright],
            spaceless: true,
            ..Options::default()
        });
        let msg = msg.text.replace('\n', "\r\n");
        let font_block_spacing = 5;
        let height = ((min_y + max_y) / 2) - font_block_spacing;
        write!(self.output, "{}{}", termion::cursor::Goto(1, height), msg).unwrap();
    }

    pub fn draw_game_over_message(&mut self, len: usize) {
        let msg = cfonts::render(Options {
            text: String::from("game|over!"),
            font: Fonts::FontHuge,
            align: Align::Center,
            ..Options::default()
        });
        let msg = msg.text.replace('\n', "\r\n");
        let prompt = format!(
            "You reached a snake length of {len}! Would you like to play again?|Press 'p' to play again, press 'q' to quit."
        );
        let msg2 = cfonts::render(Options {
            text: prompt,
            font: Fonts::FontConsole,
            align: Align::Center,
            ..Options::default()
        });
        let msg2 = msg2.text.replace('\n', "\r\n").to_uppercase();
        write!(self.output, "{}", termion::cursor::Goto(1, 1)).unwrap();
        write!(self.output, "{}{}{}", msg, msg2, color::Bg(color::Reset),).unwrap();
    }

    pub fn draw_pre_game_message(&mut self) {
        let msg1 = cfonts::render(Options {
            text: String::from("welcome to"),
            font: Fonts::FontBlock,
            align: Align::Center,
            colors: vec![Colors::Yellow, Colors::Candy],
            ..Options::default()
        });
        let msg1 = msg1.text.replace('\n', "\r\n");
        let msg2 = cfonts::render(Options {
            text: String::from("snake"),
            font: Fonts::Font3d,
            align: Align::Center,
            colors: vec![Colors::Green, Colors::Gray],
            ..Options::default()
        });
        let msg2 = msg2.text.replace('\n', "\r\n");
        write!(self.output, "{}", termion::cursor::Goto(1, 1)).unwrap();
        write!(self.output, "{}{}", msg1, msg2).unwrap();
    }

    pub fn draw_border(&mut self, xmin: u16, xmax: u16, ymin: u16, ymax: u16) {
        for i in xmin - 1..=xmax + 1 {
            for j in ymin - 1..=ymax + 1 {
                match i {
                    n if (n == xmin - 1 || n == xmax + 1) => write!(
                        self.output,
                        "{goto}{bgColor} ",
                        goto = cursor::Goto(i, j),
                        bgColor = color::Bg(color::White),
                    )
                    .unwrap(),
                    _ => (),
                };
                match j {
                    n if (n == ymin - 1 || n == ymax + 1) => write!(
                        self.output,
                        "{goto}{bgColor} ",
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

    pub fn draw_food(&mut self, food: &GridCell) {
        write!(
            self.output,
            "{goto}{bgColor}{fgColor}{food_char}{fgreset}{bgreset}",
            goto = cursor::Goto(food.x, food.y),
            bgColor = color::Bg(color::Red),
            fgColor = color::Fg(color::LightGreen),
            food_char = FOOD_CHAR,
            fgreset = color::Fg(color::Reset),
            bgreset = color::Bg(color::Reset),
        )
        .unwrap();
    }

    pub fn draw_snake(&mut self, snake: &Snake) {
        let body = &snake.body;
        let len = body.len();
        for (seg_num, segment) in body.iter().enumerate() {
            let segment_char = match seg_num {
                0 => 'S',
                num if num == len - 1 => 'e',
                1 => 'n',
                num if num == len - 2 => 'k',
                _ => 'a',
            };
            write!(
                self.output,
                "{goto}{fgColor}{bgColor}{segment_char}{reset}",
                goto = cursor::Goto(segment.x, segment.y),
                fgColor = color::Fg(color::Black),
                bgColor = color::Bg(color::Green),
                segment_char = segment_char,
                reset = color::Bg(color::Reset),
            )
            .unwrap();
        }
        let tail = &snake.old_tail;
        if let Some(x) = tail {
            self.undraw(x)
        }
    }

    fn undraw(&mut self, cell: &GridCell) {
        write!(self.output, "{} ", cursor::Goto(cell.x, cell.y)).unwrap();
    }
}
