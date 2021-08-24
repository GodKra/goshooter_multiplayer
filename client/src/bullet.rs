use std::io::Write;

use crossterm::{cursor, queue, style::{PrintStyledContent, Stylize}};

#[derive(Debug)]
pub struct Bullet {
    x: u16,
    y: u16,
}

impl Bullet {
    pub fn new(x: u16, y: u16) -> Bullet {
        Bullet { x, y: y - 1 }
    }

    pub fn move_to(&mut self, y: u16) {
        self.y = y;
    }

    pub fn draw(&self, stdout: &mut impl Write) {
        queue!(stdout, cursor::MoveTo(self.x, self.y), PrintStyledContent("o".white())).unwrap();
    }
}
