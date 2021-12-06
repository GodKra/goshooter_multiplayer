use ggez::graphics;

use common::{BULLET_UPDATE_INTERVAL, BULLET_UPDATE_MOVEMENT};

#[derive(Debug)]
pub struct Bullet {
    x: f32, // current x
    y: f32, // current y

    start_y: f32,
    final_y: f32,
    dt: f32, // delta time

    body: graphics::Mesh,
}

impl Bullet {
    pub fn new(ctx: &mut ggez::Context, x: f32, y: f32, start_y: f32, final_y: f32) -> Bullet {
        let body = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            ggez::mint::Point2::from([0.0, 0.0]),
            10.0,
            2.0,
            graphics::Color::WHITE,
        ).unwrap();

        Bullet { 
            x, 
            y,
            start_y,
            final_y,
            dt: 0.0,
            body,
        }
    }

    // pub fn x(&self) -> f32 {
    //     self.x
    // }
    
    // pub fn y(&self) -> f32 {
    //     self.y
    // }

    pub fn set_dt(&mut self, delta_t: f32) -> &mut Self {
        self.dt = delta_t;
        self
    }

    pub fn update(&mut self) -> bool {
        let v = (self.start_y-self.final_y)/((((self.start_y-self.final_y).abs()*BULLET_UPDATE_INTERVAL as f32)/BULLET_UPDATE_MOVEMENT as f32)/self.dt);
        if (v >= 0.0 && self.y >= v) || (v <= 0.0 && self.y <= (self.final_y + v)) {
            self.y -= v;
        } else {
             self.y = self.final_y;
        }

        self.y != self.final_y
    }

    // pub fn move_to(&mut self, y: f32) -> &mut Self {
    //     self.y = y;
    //     self
    // }

    pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::draw(ctx, &self.body, (ggez::mint::Point2::from([self.x, self.y]),))
    }
}