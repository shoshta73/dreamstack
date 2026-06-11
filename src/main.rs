use tokio::time::{self, Duration};

mod game;
mod player;

use game::Clock;
use player::Player;

#[tokio::main]
async fn main() {
    let _player = Player::default();
    let ticker = tokio::spawn(async move {
        let mut clock = Clock::default();
        let mut interval = time::interval(Duration::from_secs_f64(1.0 / 60.0));

        loop {
            interval.tick().await;
            clock.tick();
            println!("game time {clock}");
        }
    });

    ticker.await.expect("ticker task failed");
}
