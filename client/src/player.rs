use std::io::Write;

use crossterm::{cursor, queue, style::{PrintStyledContent, Stylize}};

pub struct Player {
    x: u16,
    y: u16,
    max_x: u16,
}

impl Player {
    pub fn new(max_width: u16, max_height: u16) -> Player {
        Player {
            x: max_width / 2,
            y: max_height - 2,
            max_x: max_width,
        }
    }

    pub fn move_right(&mut self) {
        if self.x < self.max_x - 2 {
            self.x += 1;
        }
    }
    pub fn move_left(&mut self) {
        if self.x > 1 {
            self.x -= 1;
        }
    }

    pub fn draw(&self, stdout: &mut impl Write) {
        queue!(stdout, cursor::MoveTo(self.x-1, self.y), PrintStyledContent("/^\\".green())).unwrap();
    }
}
