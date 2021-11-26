use server::Server;

mod team;
mod server;
mod player;
mod bullet;

#[tokio::main]
async fn main() {
    let mut server = Server::new(600, 600, 2, 6773).await.unwrap();
    server.start().await.unwrap();
}
