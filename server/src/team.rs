use std::{
    collections::{HashMap, VecDeque}, 
    net::TcpListener, 
    time::{self, Duration}
};

use bus::Bus;
use crossbeam::channel::{self, Receiver};
use nanoid::nanoid;

use common::{Event, Packet};
use crate::{
    bullet::Bullet, 
    player::Player
};

#[derive(Clone)]
enum BaseState {
    Healthy,
    Damaged,
}

pub struct Team {
    width: u8,
    height: u8,

    players: HashMap<u8, Player>,
    bullets: VecDeque<(String, Bullet)>, 
    enemies: VecDeque<(String, Bullet)>,
    base: Vec<BaseState>,

    bullet_ticker: Receiver<time::Instant>,
    p_bus: Option<Bus<Packet>>,
}

impl Team {
    pub fn new(width: u8, height: u8) -> Team {
        Team {
            width,
            height,
            players: HashMap::new(), 
            bullets: VecDeque::new(), 
            enemies: VecDeque::new(),
            base: vec![BaseState::Healthy; width as usize],
            
            bullet_ticker: channel::tick(Duration::from_millis(100)),
            p_bus: Some(Bus::new(1024)) 
        }
    }

    pub fn handle_team(&mut self, enemy_team: &mut Team, packet_bus: &mut Bus<Packet>) -> bool {

        let mut disconnects = Vec::new();
        // handle player events
        for (_, player) in self.players.iter_mut() {
            if let Ok(Packet::PlayerEvent{pid, event}) = player.reciever.try_recv() {
                println!("recieve: {:?}", Packet::PlayerEvent{pid, event: event.clone()});
                match event {
                    Event::MoveRight => {
                        player.move_right();
                        packet_bus.broadcast(Packet::PlayerEvent{pid, event});
                    },
                    Event::MoveLeft => {
                        player.move_left();
                        packet_bus.broadcast(Packet::PlayerEvent{pid, event});
                    }
                    Event::Fire => {
                        if player.last_fired() > Duration::from_millis(400) {
                            self.bullets.push_back((nanoid!(8), player.fire()));
                        }
                    },
                    Event::Exit => {
                        disconnects.push(pid);
                        packet_bus.broadcast(Packet::PlayerDestroy(pid));
                    },
                }   
            }
        }
        // handle disconnects
        self.players.retain(|id, _| { 
            for pid in &disconnects {
                if id == pid {
                    println!("Player {} disconnected.", pid);
                    return false
                }
            }
            true
        });
        
        // bullet update and collision checks
        if self.bullet_ticker.try_recv().is_ok() {
            let mut collisions = Vec::new();

            // bullet update
            let mut invalid_b = 0;
            for (i, (id, bullet)) in self.bullets.iter_mut().enumerate() {
                if !bullet.fly() {
                    invalid_b += 1;
                    continue;
                }
                packet_bus.broadcast(Packet::BulletPos{ id: id.to_string(), x: bullet.x(), y: bullet.y() });
                for (j, (_, enemy)) in self.enemies.iter_mut().enumerate() {
                    if bullet.collides_with(enemy) {
                        collisions.push((i, j));
                    }
                }
            }

            for _ in 0..invalid_b {
                let (id, bullet) = self.bullets.pop_front().unwrap();
                packet_bus.broadcast(Packet::BulletDestroy(id.to_string()));
                enemy_team.add_enemy(id, bullet.x(), bullet.max_y());
            }

            //collisions
            for (bullet, enemy) in collisions {
                let (bullet, _) = self.bullets.remove(bullet).unwrap();
                let (enemy, _) = self.enemies.remove(enemy).unwrap();
                packet_bus.broadcast(Packet::BulletDestroy(bullet));
                packet_bus.broadcast(Packet::BulletDestroy(enemy));
            }
            
            //enemy update
            let mut invalid_e = 0;
            for (id, enemy) in self.enemies.iter_mut() {
                if !enemy.fall() {
                    invalid_e += 1;
                    continue;
                }
                packet_bus.broadcast(Packet::BulletPos{ id: id.to_string(), x: enemy.x(), y: enemy.y() });
            }
            for _ in 0..invalid_e {
                let (id, enemy) = self.enemies.pop_front().unwrap();
                packet_bus.broadcast(Packet::BulletDestroy(id.to_string()));
                match self.base[enemy.x() as usize] {
                    BaseState::Healthy => {
                        self.base[enemy.x() as usize] = BaseState::Damaged;
                    },
                    BaseState::Damaged => {
                        return true; // game over
                    }
                }
            }
        }
        false
    }

    pub fn start_game(&mut self) {
        self.broadcast(Packet::GameStart{
            width: self.width,
            height: self.height,
            pids: self.player_ids(),
        });
    }

    pub fn add_player(&mut self, id: u8, listener: &TcpListener) {
        self.players.insert(id, Player::new(
            id, 
            self.width,
            self.height,
            &listener, 
            self.p_bus.as_mut().unwrap().add_rx()
        ));
    }

    pub fn add_enemy(&mut self, id: String, x: u8, max_y: u8) {
        self.enemies.push_back((id, Bullet::new(x, 0, max_y)));
    }

    pub fn player_ids(&self) -> Vec<u8> {
        self.players.keys().cloned().collect()
    }

    pub fn broadcast(&mut self, packet: Packet) {
        self.p_bus.as_mut().unwrap().broadcast(packet);
    }

    pub fn get_bus(&mut self) -> Bus<Packet> {
        self.p_bus.take().unwrap()
    }
}