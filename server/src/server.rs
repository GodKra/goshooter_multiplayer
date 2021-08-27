use std::{net::{SocketAddrV4, TcpListener}, thread, time::Duration};

use common::Packet;
use crate::team::Team;

pub struct Server {
    listener: TcpListener,

    player_count: u8,
    top: Team,
    bottom: Team,
}

impl Server {
    pub fn new(width: u8, height: u8, player_count: u8, port: u16) -> Server {
        Server {
            listener: TcpListener::bind::<SocketAddrV4>(format!("0.0.0.0:{}", port).parse().unwrap()).unwrap(),
            player_count,
            top: Team::new(width, height),
            bottom: Team::new(width, height),
        }
    }

    pub fn run(&mut self) {
        println!("Waiting for players.");
        for id in (0..self.player_count).step_by(2) {
            self.top.add_player(id, &self.listener);
            self.bottom.add_player(id+1, &self.listener);
        }

        self.top.start_game();
        self.bottom.start_game();

        let mut topbus = self.top.get_bus();
        let mut botbus = self.bottom.get_bus();
        let (mut echo1, mut echo2) = (topbus.add_rx(), botbus.add_rx());
        
        loop {
            if let true = self.top.handle_team(&mut self.bottom, &mut topbus) {
                topbus.broadcast(Packet::GameLost);
                botbus.broadcast(Packet::GameWon);
                println!("-- Bottom Team Wins --");
                break;
            };
            if let true = self.bottom.handle_team(&mut self.top, &mut botbus) {
                topbus.broadcast(Packet::GameWon);
                botbus.broadcast(Packet::GameLost);
                println!("-- Top Team Wins --");
                break;
            };
            if let Ok(packet) = echo1.try_recv() {
                println!("sent: {:?}", packet);
            }
            if let Ok(packet) = echo2.try_recv() {
                println!("sent: {:?}", packet)
            }
            
            thread::sleep(Duration::from_millis(1));
        }
        thread::sleep(Duration::from_secs(1));
    }
}
