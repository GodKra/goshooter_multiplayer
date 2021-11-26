#[derive(Debug)]
pub struct Bullet {
    x: u32,
    y: u32,
    max_y: u32,
}

impl Bullet {
    pub fn new(x: u32, y: u32, max_y: u32) -> Self {
        Bullet { x, y, max_y }
    }
    pub fn fly(&mut self) -> bool {
        if self.y >= 10 { 
            self.y -= 10 
        } else {
            self.y = 0
        }
        self.y > 0
    }
    pub fn fall(&mut self) -> bool {
        if self.y < self.max_y { self.y += 10 }
        self.y < self.max_y
    }
    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    pub fn max_y(&self) -> u32 {
        self.max_y
    }
    pub fn collides_with(&self, other: &Self) -> bool {
        self.x == other.x() && self.y < other.y()+1
    }
}