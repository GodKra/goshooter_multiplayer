pub struct Bullet {
    x: u8,
    y: u8,
    max_y: u8,
}

impl Bullet {
    pub fn new(x: u8, y: u8, max_y: u8) -> Self {
        Bullet { x, y, max_y }
    }
    pub fn fly(&mut self) -> bool {
        if self.y > 0 { self.y -= 1 }
        self.y > 0
    }
    pub fn fall(&mut self) -> bool {
        if self.y < self.max_y { self.y += 1 }
        self.y < self.max_y
    }
    pub fn x(&self) -> u8 {
        self.x
    }
    pub fn y(&self) -> u8 {
        self.y
    }
    pub fn max_y(&self) -> u8 {
        self.max_y
    }
    pub fn collides_with(&self, other: &Self) -> bool {
        self.x == other.x() && self.y < other.y()+1
    }
}