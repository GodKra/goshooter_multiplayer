mod client;
mod bullet;
mod events;
mod player;

use std::{io, net::ToSocketAddrs};
fn main() {
    let ip = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Please enter ip: ");
        let mut ip = String::new();
        io::stdin().read_line(&mut ip).unwrap();
        ip
    });
    let stdout = io::stdout();
    let stdout = stdout.lock();
    
    let mut game = client::Game::new(ip.trim().to_socket_addrs().unwrap().next().unwrap(), stdout);

    game.start();
}
