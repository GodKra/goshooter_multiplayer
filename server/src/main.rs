mod server;
mod player;
mod bullet;
mod team;

use structopt::StructOpt;

use server::Server;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long, default_value = "2", help = "Total player count")]
    count: u8,
    #[structopt(short, long, default_value = "50", help = "Game width")]
    width: u8,
    #[structopt(short, long, default_value = "25", help = "Game height")]
    height: u8,
    #[structopt(short, long, default_value = "6773", help = "Server port")]
    port: u16,
}

fn main() {
    let args = Cli::from_args();
    let mut server = Server::new(args.width, args.height, args.count, args.port);
    server.run();
    println!("Server closed.");
}
