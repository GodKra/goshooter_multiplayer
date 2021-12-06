use common::BULLET_UPDATE_MOVEMENT;

#[derive(Clone, Debug)]
pub struct Bullet {
    x: u32,
    y: u32,
    radius: u32,
    max_y: u32,
}

impl Bullet {
    pub fn new(x: u32, y: u32, max_y: u32) -> Self {
        Bullet { x, y, radius: 10, max_y }
    }
    pub fn fly(&mut self) -> bool {
        if self.y >= BULLET_UPDATE_MOVEMENT { 
            self.y -= BULLET_UPDATE_MOVEMENT;
        } else {
            self.y = 0;
        }
        self.y > 0
    }
    pub fn fall(&mut self) -> bool {
        if self.y < self.max_y { 
            self.y += BULLET_UPDATE_MOVEMENT;
        } else {
            self.y = self.max_y;
        }
        self.y < self.max_y
    }
    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    
    // circle collision
    pub fn collides_with(&self, other: &Self) -> bool {
        let (r1, r2) = (self.radius as i32, other.radius as i32);
        let (x1, y1) = (self.x as i32, self.y as i32);
        let (x2, y2) = (other.x as i32, other.y as i32);
        (r1+r2)*(r1+r2) > (x2-x1)*(x2-x1) + (y2-y1)*(y2-y1)
    }
}