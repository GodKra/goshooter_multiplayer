use std::{
    collections::{HashMap, VecDeque}, 
    sync::Arc,
    time::Duration
};

use tokio::{
    net::TcpListener, 
    sync::{
        broadcast, 
        mpsc::{self, Receiver, Sender},
        Mutex,
    }, 
    time,
};

use nanoid::nanoid;

use common::*;

use crate::{bullet::Bullet, player::Player};


//#[derive(Clone)]
// enum BaseState {
//     Healthy,
//     Damaged,
// }

pub struct Team {
    width: u32,
    height: u32,

    state: Arc<Mutex<State>>,
    score: u32,
    //base: Vec<BaseState>,
    
    bullet_ticker: time::Interval,
    
    p_recv: Option<broadcast::Receiver<Packet>>,
    tcomms_recv: Option<Receiver<Packet>>, // inter-team comms reciever; recieve bullet hits
    tcomms_send: Sender<Packet>, // inter-team comms sender; send bullet hits
    enemy_recv: Option<Receiver<(String, Bullet)>>, // recieve enemy bullets
    enemy_send: Sender<(String, Bullet)>, // send enemy bullets
}

struct State {
    players: HashMap<String, Player>,
    bullets: VecDeque<(String, Bullet)>, 
    enemies: VecDeque<(String, Bullet)>,
    p_sender: Option<broadcast::Sender<Packet>>,
}

impl State {
    pub fn new() -> (Self, broadcast::Receiver<Packet>) {
        let (tx, rx) = broadcast::channel(1024);
        (State {
            players: HashMap::new(), 
            bullets: VecDeque::new(), 
            enemies: VecDeque::new(),
            p_sender: Some(tx),
        }, rx)
    }
}


impl Team {
    pub fn new(width: u32, height: u32) -> Team {
        let (enemy_tx, enemy_rx) = mpsc::channel(256);
        let (hit_tx, hit_rx) = mpsc::channel(32);
        let (state, p_rx) = State::new();
        
        let mut bullet_ticker = time::interval(Duration::from_millis(BULLET_UPDATE_INTERVAL));
        bullet_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        Team {
            width,
            height,
            state: Arc::new(Mutex::new(state)),
            score: 0,
            //base: vec![BaseState::Healthy; width as usize],
            
            bullet_ticker, // bullet speed
            p_recv: Some(p_rx),
            enemy_recv: Some(enemy_rx),
            enemy_send: enemy_tx,
            tcomms_send: hit_tx,
            tcomms_recv: Some(hit_rx),
            
        }
    }

    pub async fn handle_team(&mut self) -> Result<crate::server::GameResult> {
        // State updates
        let mut p_recv = self.p_recv.take().unwrap();
        let state = self.state.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(packet) = p_recv.recv().await {
                    let mut state = state.lock().await;
                    match packet {
                        Packet::PlayerDestroy(pid) => {
                            state.players.remove(&pid);
                        }
                        Packet::PlayerEvent{ pid, event } => {
                            if let PlayerEvent::Fire = event {
                                let player = state.players.get_mut(&pid);
                                if player.is_none() {
                                    println!("Invalid player id received: {}", pid);
                                    continue;
                                }
                                let player = player.unwrap();
                                if player.last_fired() > Duration::from_millis(PLAYER_FIRE_INTERVAL) {
                                    let bullet = player.fire();
                                    let id = nanoid!(BULLET_ID_LEN);
                                    state.p_sender.as_mut().unwrap().send(Packet::BulletCreate{ id: id.clone(), x: bullet.x(), y: bullet.y() }).unwrap();
                                    state.bullets.push_back((id, bullet));
                                }
                            }
                        }
                        Packet::PlayerPos{ pid, x, y } => {
                            let player = state.players.get_mut(&pid);
                            if player.is_none() {
                                println!("Invalid player id received: '{}'", pid);
                                continue;
                            }
                            let player = player.unwrap();
                            if player.last_updated() > Duration::from_millis(PLAYER_UPDATE_INTERVAL) {
                                player.move_to(x, y);
                            }
                        }
                        Packet::GameWon | Packet::GameLost => {
                            println!("shutting down state update loop");
                            return
                        }
                        _ => (),
                    }
                }
            }
        });
        // bullet & enemy updates & collisions
        let enemy_recv = self.enemy_recv.as_mut().unwrap();
        let tcomms_recv = self.tcomms_recv.as_mut().unwrap();
        loop {
            tokio::select! {
                _ = self.bullet_ticker.tick() => {
                    let state = &mut *self.state.lock().await;
                    let sender = state.p_sender.as_mut().unwrap().clone();

                    let mut bullets_invalid = 0;
                    let mut collisions: Vec<(usize, usize)> = Vec::new();
                    // update bullets
                    for (i, (id, bullet)) in state.bullets.iter_mut().enumerate() {
                        if !bullet.fly() { // bullet reached top
                            println!("bullet {} transfered", id);
                            self.enemy_send.send((id.clone(), bullet.clone())).await?;
                            sender.send(Packet::BulletDestroy(id.to_string())).unwrap();
                            bullets_invalid += 1;
                            continue;
                        }
                        //check collisions
                        for (j, (_, enemy)) in state.enemies.iter_mut().enumerate() {
                            if bullet.collides_with(enemy) {
                                collisions.push((i, j));
                            }
                        }
                    }
                    // handle collisions
                    for (bullet, enemy) in collisions {
                        println!("collision: bullet {}, enemy: {}, bullets_invalid: {}", bullet, enemy, bullets_invalid);
                        let (bullet, _) = state.bullets.remove(bullet).unwrap();
                        let (enemy, _) = state.enemies.remove(enemy).unwrap();
                        sender.send(Packet::BulletDestroy(bullet)).unwrap();
                        sender.send(Packet::EnemyDestroy(enemy)).unwrap();
                    }
                    // remove bullets out of bounds
                    for _ in 0..bullets_invalid {
                        state.bullets.pop_front();
                    }
                    // update enemies
                    let mut enemies_invalid = 0;
                    for (id, enemy) in state.enemies.iter_mut() {
                        if !enemy.fall() { // enemy reached bottom
                            println!("enemy {} hit", id);
                            enemies_invalid += 1;
                            sender.send(Packet::EnemyDestroy(id.to_string())).unwrap();
                            sender.send(Packet::EnemyHit).unwrap();
                            self.tcomms_send.send(Packet::EnemyHit).await?;
                        }
                    }
                    // remove enemies out of bounds
                    for _ in 0..enemies_invalid {
                        state.enemies.pop_front();
                    }
                }
                Some((id, enemy)) = enemy_recv.recv() => {
                    let mut state = self.state.lock().await;
                    state.p_sender.as_mut().unwrap().send(Packet::EnemyCreate{ id: id.clone(), x: enemy.x(), y: enemy.y() }).unwrap();
                    state.enemies.push_back((id, enemy));
                }
                Some(packet) = tcomms_recv.recv() => { // recieve from other team
                    let mut state = self.state.lock().await;
                    let sender = state.p_sender.as_mut().unwrap();

                    match packet {
                        Packet::EnemyHit => {
                            sender.send(Packet::BulletHit).unwrap();

                            self.score += 1;
                            if self.score >= GAME_END_SCORE {
                                self.tcomms_send.send(Packet::GameWon).await?;
                                sender.send(Packet::GameWon).unwrap();
                                println!("won");
                                return Ok(crate::server::GameResult::Won) // won
                            }
                        }
                        Packet::GameWon => { // enemy other sent won
                            sender.send(Packet::GameLost).unwrap();
                            println!("lost");
                            return Ok(crate::server::GameResult::Lost) // lost
                        }
                        _ => (),
                    }   
                }
            }
        }
    }

    

    pub async fn add_player(&mut self, listener: &TcpListener) -> Result<()> {
        let mut state = self.state.lock().await;
        let (id, player) = Player::new(
            self.width,
            self.height,
            &listener,
            state.p_sender.as_mut().unwrap().clone(),
            state.p_sender.as_mut().unwrap().subscribe()
        ).await?;
        state.players.insert(id, player);
        Ok(())
    }

    pub async fn get_pids(&self) -> Vec<String> {
        let state = self.state.lock().await;
        state.players.keys().cloned().collect()
    }

    pub async fn start_game(&mut self) {
        self.broadcast(Packet::GameInfo{
            width: self.width,
            height: self.height,
            pids: self.get_pids().await,
        }).await;
    }

    pub async fn broadcast(&mut self, packet: Packet) {
        let mut state = self.state.lock().await;
        state.p_sender.as_mut().unwrap().send(packet).unwrap();
    }

    pub fn get_enemy_rx(&mut self) -> Receiver<(String, Bullet)> {
        return self.enemy_recv.take().unwrap();
    }
    pub fn set_enemy_rx(&mut self, rx: Receiver<(String, Bullet)>) {
        self.enemy_recv = Some(rx);
    }
    pub fn get_tcomms_rx(&mut self) -> Receiver<Packet> {
        return self.tcomms_recv.take().unwrap();
    }
    pub fn set_tcomms_rx(&mut self, rx: Receiver<Packet>) {
        self.tcomms_recv = Some(rx);
    }
}