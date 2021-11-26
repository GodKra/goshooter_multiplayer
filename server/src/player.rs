use std::{time::{self, Duration}};

use tokio::{io::{AsyncWriteExt, BufReader}, net::TcpListener, sync::{broadcast}};

use common::*;

use crate::{bullet::Bullet};

pub struct Player {
    x: u32,
    _max_x: u32,
    max_y: u32,
    last_fired: time::Instant,
}

impl Player {
    pub async fn new(
                    max_x: u32, 
                    max_y: u32, 
                    listener: &TcpListener, 
                    sender: broadcast::Sender<Packet>,
                    mut reciever: broadcast::Receiver<Packet>
                ) -> Result<(String, Player)> {
        let (stream, _) = listener.accept().await.unwrap();
        let (stream_r, mut stream_w) = stream.into_split();
        let mut stream_r = BufReader::new(stream_r);

        let id = if let Ok(Some(Packet::PlayerJoin(id))) = Packet::async_read_from(&mut stream_r).await {
            id
        } else {
            return Err(String::from("Player id not recieved").into());
        };
        
        let pid = id.clone();
        tokio::spawn(async move {
            // Packet handling 
            loop {
                tokio::select! {
                    Ok(packet) = reciever.recv() => {
                        match packet {
                            Packet::PlayerEvent { .. } => (),
                            _ =>  { stream_w.write(&packet.parse()).await.unwrap(); },
                        }
                    }
                    Ok(Some(packet)) = Packet::async_read_from(&mut stream_r) => {
                        match packet {
                            Packet::PlayerEvent{ event, .. } => match event {
                                PlayerEvent::Fire => {
                                    sender.send(Packet::PlayerEvent{pid: pid.clone(), event}).unwrap();
                                },
                                PlayerEvent::Exit => {
                                    sender.send(Packet::PlayerDestroy(pid.to_string())).unwrap();
                                },
                            },
                            Packet::PlayerPos { pid, x, y, } => {
                                sender.send(Packet::PlayerPos { pid: pid.to_string(), x, y }).unwrap();
                            },
                            _ => (),
                        }
                    }
                }
            }
        });

        Ok((id, Player { x: max_x/2, _max_x: max_x, max_y, last_fired: time::Instant::now()}))
    }
    pub fn move_to(&mut self, x: u32) {
        self.x = x;
    }
    pub fn fire(&mut self) -> Bullet {
        self.last_fired = time::Instant::now();
        Bullet::new(self.x, self.max_y - 2, self.max_y)
    }
    pub fn last_fired(&self) -> Duration {
        self.last_fired.elapsed()
    }
}