use ggez::{
    GameResult, 
    graphics,
};

use mint::Point2;

#[derive(Debug)]
pub struct Player {
    x: f32,
    y: f32,

    xx: f32, // desired x
    yy: f32, // desired y
    v: f32, // velocity

    body: graphics::Image,
    name: graphics::Text,
}

impl Player {
    pub fn new(ctx: &mut ggez::Context, name: &str) -> Player {
        //let img = include_bytes!("../test2.png");
        //let body = graphics::Image::from_bytes(ctx, img).unwrap();
        let body = graphics::Image::solid(ctx, 50, graphics::Color::WHITE).unwrap();
        let name = graphics::Text::new(name);

        Player {
            x: 0.0,
            y: 500.0,
            xx: 0.0,
            yy: 0.0,
            v: 0.0,
            body,
            name,
        }
    }

    pub fn move_dx(&mut self, dx: f32) {
        self.x += dx;
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.xx = x;
        self.yy = y;
    }

    pub fn update(&mut self) -> bool {
        if (self.v > 0.0 && self.dx() > 0.0) || (self.v < 0.0 && self.dx() < 0.0) {
            self.move_dx(self.v);
        } else  {
            return false
        }
        true
    }

    pub fn set_v(&mut self, v: f32) {
        self.v = v;
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn mid_x(&self) -> f32 {
        self.x + (self.body.height()/2) as f32
    }

    pub fn get_actual_x(&self, x: f32) -> f32 {
        x - (self.body.height()/2) as f32
    }
    
    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn dx(&self) -> f32 {
        return self.xx - self.x
    }
    
    pub fn draw(&self, ctx: &mut ggez::Context) -> GameResult {
        graphics::draw(ctx, &self.body, (Point2::from([self.x, self.y]),))?;
        let center_bottom = [
            self.mid_x() - (self.name.width(ctx)/2.0), 
            self.y + (self.body.height()) as f32
        ];
        graphics::draw(ctx, &self.name, (Point2::from(center_bottom),))
    }
}
