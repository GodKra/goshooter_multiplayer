use std::{
    collections::HashMap, 
    io::{self, Read, Write}, 
    net::{SocketAddrV4, TcpStream}, 
    thread, time::Duration
};

use crossbeam::channel;
use crossterm::{cursor, execute, queue, style::Print, terminal};

use crate::{
    bullet::Bullet,
    events::EventHandler,
    player::Player
};

use common::{Packet, Event};

pub struct Game<W: Write> {
    width: u16,
    height: u16,

    stdout: W,
    stream: TcpStream,

    id: u8,
    player: Player,
    players: HashMap<u8, Player>,
    bullets: HashMap<String, Bullet>,
    base: Vec<String>,
}

impl<W: Write> Game<W> {
    pub fn new(ip: SocketAddrV4, stdout: W) -> Game<W> {
        let mut stream = TcpStream::connect(ip).unwrap();

        // get self id
        let id = if let Packet::PlayerCreate(pid) = Packet::read_from(&mut stream).unwrap().unwrap() {
            pid
        } else { panic!("no id recieved") };
        println!("Connected as Player {}, waiting to start.", id);
        // get game info
        let (width, height, players) = if let Packet::GameStart { width, height, pids } = Packet::read_from(&mut stream).unwrap().unwrap() {
            let players: HashMap<u8, Player> = pids.into_iter()
                .filter(|key| { *key != id } )
                .map(|key| {
                    (key, Player::new(width as u16, height as u16))
                }).collect();
            (width as u16, height as u16, players)
        } else { panic!("no game info recieved") };

        stream.set_nonblocking(true).unwrap();
        Game {
            stdout,
            stream,
            width,
            height,
            id, 
            player: Player::new(width, height),
            players,
            bullets: HashMap::new(),
            base: vec!["━".to_string(); width as usize - 2],
        }
    }

    pub fn start(&mut self) {
        let (sender, reciever) = channel::unbounded();
        EventHandler::handle_events(sender);

        terminal::enable_raw_mode().unwrap();

        execute!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide
        ).unwrap();
        
        loop {
            if let Ok(Some(packet)) = Packet::read_from(&mut self.stream) {
                match packet {
                    Packet::PlayerDestroy(id) => {
                        self.players.remove(&id);
                    },
                    Packet::PlayerEvent { pid, event } => {
                        match event {
                            Event::MoveRight => {
                                if let Some(player) = self.players.get_mut(&pid) {
                                    player.move_right();
                                }
                            },
                            Event::MoveLeft => {
                                if let Some(player) = self.players.get_mut(&pid) {
                                    player.move_left();
                                }
                            },
                            _ => (),
                        }
                    },
                    Packet::BulletPos { id, x, y } => {
                        self.bullets.entry(id)
                                    .or_insert_with(|| Bullet::new(x as u16, y as u16))
                                    .move_to(y as u16);
                        if y == self.height as u8 - 1 {
                            self.base[x as usize - 1] = " ".to_string();
                        }
                    },
                    Packet::BulletDestroy(id) => {
                        self.bullets.remove(&id);
                    },
                    Packet::GameWon => {
                        self.game_over("You won");
                        return;
                    }, 
                    Packet::GameLost => {
                        self.game_over("You lost");
                        return;
                    },
                    _ => (),
                }
            } else if let Ok(event) = reciever.try_recv() {
                self.stream.write_all(&Packet::PlayerEvent{pid: self.id, event: event.clone()}.parse()).unwrap();
                match event {
                    Event::MoveRight => self.player.move_right(),
                    Event::MoveLeft => self.player.move_left(),
                    Event::Exit => break,
                    _ => (),
                }
            } else {
                continue;
            }
            self.update_screen();
            thread::sleep(Duration::from_millis(1));
        }
        self.game_over("You quit");
    }

    fn update_screen(&mut self) {
        self.draw_border();

        self.player.draw(&mut self.stdout);
        for (_, player) in self.players.iter() {
            player.draw(&mut self.stdout);
        }
        for (_, bullet) in self.bullets.iter() {
            bullet.draw(&mut self.stdout);
        }

        self.stdout.flush().unwrap();
    }

    fn draw_border(&mut self) {
        let (width, height) = (self.width, self.height);
        // top
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            Print(format!("┏{}┓", "━".repeat(width as usize - 2))),
        ).unwrap();
        // bottom
        let base = self.base.join("");
        queue!(
            self.stdout,
            cursor::MoveTo(0, height-1),
            Print(format!("┗{}┛", base)),
        ).unwrap();
        // sides
        for y in 1..self.height-1 {
            queue!(
                self.stdout,
                cursor::MoveTo(0, y),
                Print(format!("┃{}┃", " ".repeat(width as usize - 2))),
            ).unwrap();
        }
    }

    fn game_over(&mut self, text: &str) {
        let (width, height) = (self.width, self.height);
        execute!(
            self.stdout,
            cursor::MoveTo(width / 2 - text.len() as u16, height / 2),
            Print(text),
            cursor::MoveTo(width / 2 - 5, height / 2 + 1),
            //Print(format!("Score: {}", score))
        ).unwrap();

        self.stdout.flush().unwrap();
        
        thread::sleep(Duration::from_secs(1));
        let _ = io::stdin().read(&mut [0u8]).unwrap();
    }
}

impl<W: Write> Drop for Game<W> {
    fn drop(&mut self) {
        execute!(
            self.stdout, 
            cursor::Show, 
            cursor::MoveTo(0, 0), 
            terminal::Clear(terminal::ClearType::All)
        ).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}