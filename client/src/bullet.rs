use ggez::graphics;

#[derive(Debug)]
pub struct Bullet {
    x: f32,
    y: f32,

    yy: f32, // final y
    v: f32, // velocity

    body: graphics::Mesh,
}

impl Bullet {
    pub fn new(ctx: &mut ggez::Context, x: f32, y: f32, final_y: f32) -> Bullet {
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
            yy: final_y,
            v: 0.0,
            body,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }
    
    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn set_v(&mut self, v: f32) {
        self.v = v;
    } // y / 180.0, // ((600.0*50.0/10.0)/(1.0/60.0 * 1000.0)) = ((total distance * time) / distance per time ms)/(seconds per frame * 1000)

    pub fn update(&mut self) -> bool {
        if (self.v > 0.0 && self.y >= self.v) || (self.v < 0.0 && self.y <= (self.yy-self.v)) {
            self.y -= self.v
        } else {
            self.y = 0.0;
        }

        self.y != self.yy
    }

    pub fn move_to(&mut self, y: f32) {
        self.y = y;
    }

    pub fn draw(&self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::draw(ctx, &self.body, (ggez::mint::Point2::from([self.x, self.y]),))
    }
}