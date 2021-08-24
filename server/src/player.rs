use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    time::{self, Duration},
    thread,
};

use crossbeam::channel::{self, Receiver, Sender};
use bus::{self, BusReader};

use common::{Event, Packet};
use crate::bullet::Bullet;

pub struct Player {
    pub reciever: Receiver<Packet>,
    x: u8,
    max_x: u8,
    max_y: u8,
    last_fired: time::Instant,
}

impl Player {
    pub fn new(id: u8, max_x: u8, max_y: u8, listener: &TcpListener, reciever: BusReader<Packet>) -> Player {
        let (stream, _) = listener.accept().unwrap();
        stream.set_nonblocking(true).unwrap();
        let (tx, rx) = channel::bounded(1024);
        println!("Player {} connected.", id);
        Player::handle_player(id, tx, reciever, stream);

        Player { reciever: rx, x: max_x/2, max_x, max_y, last_fired: time::Instant::now() }
    }

    fn handle_player(id: u8, sender: Sender<Packet>, mut reciever: BusReader<Packet>, mut stream: TcpStream) {
        stream.write_all(&Packet::PlayerCreate(id).parse()).unwrap(); // send id

        thread::spawn(move || loop {
            // recieve from bus
            if let Ok(packet) = reciever.try_recv() {
                if let Packet::PlayerDestroy(pid) = packet.clone() {
                    if pid == id {
                        return; // self destroyed
                    }
                }

                if stream.write_all(&packet.parse()).is_err() {
                    println!("Player {} write error. Disconnecting", id);
                    sender.send(Packet::PlayerEvent{ pid: id, event: Event::Exit }).unwrap();
                    return;
                }
            }
            // recieve from stream
            if let Ok(Some(packet)) = Packet::read_from(&mut stream) {
                sender.send(packet).unwrap();
            }
            thread::sleep(Duration::from_millis(1));
        });
    }

    pub fn move_right(&mut self) {
        if self.x < self.max_x - 2 {
            self.x += 1;
        }
    }
    pub fn move_left(&mut self) {
        if self.x > 1 {
            self.x -= 1;
        }
    }
    pub fn fire(&mut self) -> Bullet {
        self.last_fired = time::Instant::now();
        Bullet::new(self.x, self.max_y - 2, self.max_y)
    }
    pub fn last_fired(&self) -> Duration {
        self.last_fired.elapsed()
    }
}