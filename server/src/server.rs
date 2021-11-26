use tokio::net::TcpListener;
use common::*;

use crate::team::Team;

#[derive(Debug)]
pub enum GameResult {
    Won,
    Lost,
}

// enum Event {
//     UpdatePlayer(String, u32, u32),
//     RemovePlayer(String),
//     AddBullet(Bullet),
//     UpdateBullet(String),
//     RemoveBullet(String),
//     AddEnemy(Bullet),
// }

pub struct Server {
    listener: TcpListener,

    max_players: u8,
    top: Team,
    bottom: Team,
}


impl Server {
    pub async fn new(width: u32, height: u32, max_players: u8, port: u16) -> Result<Server> {
        let (mut top, mut bottom) = (Team::new(width, height), Team::new(width, height));
        Self::swap_enemy_channels(&mut top, &mut bottom);
        
        Ok(Server {
            listener: TcpListener::bind(format!("0.0.0.0:{}", port)).await?,
            max_players,
            top,
            bottom,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        for _ in (0..self.max_players).step_by(2) {
           self.top.add_player(&self.listener).await?;
           self.bottom.add_player(&self.listener).await?;
        }

        self.top.start_game().await;
        self.bottom.start_game().await;

        let r = tokio::join!(self.top.handle_team(), self.bottom.handle_team());
        
        println!("{:?}", r);
        Ok(())
    }

    fn swap_enemy_channels(t1: &mut Team, t2: &mut Team) {
        let t1_rx = t1.get_enemy_rx();
        t1.set_enemy_rx(t2.get_enemy_rx());
        t2.set_enemy_rx(t1_rx);
    }
}
