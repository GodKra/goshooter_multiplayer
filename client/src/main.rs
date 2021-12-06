mod player;
mod bullet;

use std::{collections::HashMap, io::Write, net::TcpStream, time::{self, Instant}};

use ggez::*;

use mint::Point2;
use player::Player;
use bullet::Bullet;
use common::*;

const MOVE_SPEED_PX: f32 = 8.0;

fn main() {
    let name = std::env::args().nth(1).unwrap_or("test".to_string());
    //let c = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new("goshooter", "test")
        .window_setup(ggez::conf::WindowSetup { 
            title: "GoShooter".to_string(), 
            samples: ggez::conf::NumSamples::One, 
            vsync: true, 
            icon: "".to_string(), 
            srgb: true 
        })
        .window_mode(ggez::conf::WindowMode { 
            width: 600.0, 
            height: 600.0, 
            maximized: false, 
            fullscreen_type: ggez::conf::FullscreenType::Windowed, 
            borderless: false, 
            min_width: 600.0, 
            min_height: 600.0, 
            max_width: 600.0, 
            max_height: 600.0, 
            resizable: false, 
            visible: true, 
            resize_on_scale_factor_change: false,
        })
        .build()
        .unwrap();

    let state = State::new(&mut ctx, &name).unwrap();
    event::run(ctx, event_loop, state);
}

#[derive(PartialEq, Clone, Copy)]
enum GameStatus {
    RUNNING,
    WON,
    LOST,
}

struct State {
    stream: TcpStream,

    width: f32,
    height: f32,
    game_status: GameStatus,

    players: HashMap<String, Player>,
    bullets: HashMap<String, Bullet>,
    pos_ticker: crossbeam::channel::Receiver<Instant>,
    last_fired: time::Instant,

    name: String,
    player: Player,
    score: u32,
    health_left: u32,
    
    // player controls
    move_r: f32,
    move_l: f32,
    moved: bool,
    fire: bool,
}

impl State {
    pub fn new(ctx: &mut Context, name: &str) -> Result<State> {
        let name_len = name.len();
        let name = if name_len > 8 { // make name 8 chars long
            name[..8].to_string()
        } else if name_len < 8 {
            name.to_string() + &"\0".repeat(8-name_len)
        } else {
            name.to_string()
        };

        let mut stream = TcpStream::connect("127.0.0.1:6773")?;
        stream.write_all(&Packet::PlayerJoin(name.to_string()).parse())?;

        println!("Connected as {}, waiting to start.", name.trim());

        // get current game information
        let (width, height, players) = 
            if let Packet::GameInfo { width, height, pids } = Packet::read_from(&mut stream)?.unwrap() {
                let players: HashMap<String, Player> = pids.into_iter()
                    .filter(|key| { *key != name } )
                    .map(|key| {
                        (key.clone(), Player::new(ctx, &key))
                    }).collect();
                (width as f32, height as f32, players)
            } else { 
                return Err(String::from("Invalid packet recieved").into());
            };
        stream.set_nonblocking(true).unwrap();
        println!("{:?}", players);

        let player = Player::new(ctx, &name);

        Ok(State {
            stream,
            width,
            height,
            game_status: GameStatus::RUNNING,
            players,
            bullets: HashMap::new(),
            pos_ticker: crossbeam::channel::tick(std::time::Duration::from_millis(100)),
            last_fired: time::Instant::now(),
            name,
            player,
            score: 0,
            health_left: GAME_END_SCORE,
            move_r: 0.0,
            move_l: 0.0,
            moved: false,
            fire: false,
        })
    }
}

impl ggez::event::EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        let dt = 1.0/DESIRED_FPS as f32 * 1000.0;
        //let dt = timer::delta(ctx).as_millis() as f32;
        //let dt = timer::average_delta(ctx).as_millis() as f32;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            // update self
            let dx = self.move_r - self.move_l;
            if dx != 0.0 {
                self.player.move_dx(self.move_r-self.move_l);
                self.moved = true;
            }
            if let Ok(_) = self.pos_ticker.try_recv() {
                if self.moved {
                    self.stream.write_all(&Packet::PlayerPos{
                        pid: self.name.clone(), 
                        x: self.player.mid_x() as u32, 
                        y: self.player.y() as u32
                    }.parse()).unwrap();
                    self.moved = false;
                }
            }
            // spawn bullets
            if self.fire && self.last_fired.elapsed() > time::Duration::from_millis(PLAYER_FIRE_INTERVAL) {
                self.stream.write_all(&Packet::PlayerPos{
                    pid: self.name.clone(), 
                    x: self.player.mid_x() as u32, 
                    y: self.player.y() as u32
                }.parse()).unwrap();
                self.stream.write_all(&Packet::PlayerEvent{pid: self.name.clone(), event: PlayerEvent::Fire}.parse()).unwrap();
                self.last_fired = time::Instant::now();
            }

            // update others
            for (_, player) in self.players.iter_mut() {
                player.update();
                player.set_dt(dt);
            }

            // update bullets & enemies
            for (_, bullet) in self.bullets.iter_mut() {
                bullet.update();
                bullet.set_dt(dt);
            }

            // handle packets
            if let Ok(Some(packet)) = Packet::read_from(&mut self.stream) {
                println!("recv: {:?}", packet);
                match packet {
                    Packet::PlayerDestroy(pid) => {
                        self.players.remove(&pid);
                    },
                    Packet::PlayerPos { pid, x, y  } => {
                        if let Some(player) = self.players.get_mut(&pid) {
                            let x = player.get_actual_x(x as f32);
                            player.set_pos(x, y as f32)
                                  .set_dt(dt);
                        }
                    },
                    Packet::BulletCreate { id, x, y } => {
                        self.bullets.entry(id)
                                    .or_insert_with(|| Bullet::new(ctx, x as f32, y as f32, y as f32, 0.0))
                                    .set_dt(dt);  
                    },
                    Packet::BulletDestroy(id) => {
                        println!("bullet {} destroy", id);
                        self.bullets.remove(&id);
                    },
                    Packet::EnemyCreate { id, x, y } => {
                        let final_y = self.height;
                        self.bullets.entry(id)
                                    .or_insert_with(|| Bullet::new(ctx, x as f32, y as f32, y as f32, final_y))
                                    .set_dt(dt);
                    },
                    Packet::EnemyDestroy(id) => {
                        println!("enemy {} destroy", id);
                        self.bullets.remove(&id);
                    },
                    Packet::BulletHit => {
                        self.score += 1;
                    },
                    Packet::EnemyHit => {
                        self.health_left -= 1;
                    }
                    Packet::GameWon => {
                        //self.won = Some(true);
                        println!("WON");
                        self.game_status = GameStatus::WON;
                        //ggez::event::quit(ctx);
                    }, 
                    Packet::GameLost => {
                        //self.won = Some(true);
                        println!("LOST");
                        self.game_status = GameStatus::LOST;
                        //ggez::event::quit(ctx);
                    },
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        if self.game_status != GameStatus::RUNNING {
            self.game_over(ctx, self.game_status)?;
            return Ok(());
        }
        
        // draw scores
        let scores = format!("Health: {}/{}\nScore: {}", self.health_left, GAME_END_SCORE, self.score);
        graphics::draw(ctx, &graphics::Text::new(scores), (Point2::from([0.0,0.0]),)).unwrap();

        // draw objects
        self.player.draw(ctx)?;

        for (_, player) in self.players.iter() {
            player.draw(ctx)?;
        }
        for (_, bullet) in self.bullets.iter() {
            bullet.draw(ctx)?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: event::KeyCode, _keymods: event::KeyMods, _repeat: bool) {
        let shift = if _keymods == event::KeyMods::SHIFT { // speed increase on shift
            2.5
        } else {
            1.0
        };

        match keycode {
            event::KeyCode::Right => self.move_r = shift*MOVE_SPEED_PX,
            event::KeyCode::Left => self.move_l = shift*MOVE_SPEED_PX,
            event::KeyCode::Up => self.fire = true,
            _ => (),
        }
        
    }

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: event::KeyCode, _keymods: event::KeyMods) {
        match _keycode {
            event::KeyCode::Right => self.move_r = 0.0,
            event::KeyCode::Left => self.move_l = 0.0,
            event::KeyCode::Up => self.fire = false,
            _ => (),
        }
    }
}

impl State {
    fn game_over(&mut self, ctx: &mut Context, game_status: GameStatus) -> GameResult {
        let text = graphics::Text::new(if game_status == GameStatus::WON {
            "Your team won"
        } else {
            "Your team lost"
        });
        let center = [self.width/2.0 - text.width(ctx) as f32/2.0, self.height/2.0];
        graphics::draw(ctx, &text, (Point2::from(center),))?;
        graphics::present(ctx)?;
        ggez::timer::sleep(std::time::Duration::from_secs(5));
        ggez::event::quit(ctx);
        Ok(())
    }
}