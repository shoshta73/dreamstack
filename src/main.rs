use tokio::time::{self, Duration};

mod game;
mod player;

use game::{Clock, level_0};
use player::Player;
use tracing::info;
use tracing_subscriber::{
    EnvFilter, Layer, fmt::layer, layer::SubscriberExt, registry, util::SubscriberInitExt,
};

#[tokio::main]
async fn main() {
    let terminal = layer()
        .with_ansi(true)
        .with_filter(if cfg!(debug_assertions) {
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"))
        } else {
            EnvFilter::new("info")
        });

    registry().with(terminal).init();

    info!("Game Started");
    let _player = Player::default();
    let level = level_0();
    info!(
        level = level.number,
        employers = level.employers.len(),
        "Level loaded"
    );

    for employer in &level.employers {
        for job in &employer.jobs {
            info!(
                employer = employer.name,
                job = job.name,
                hourly_pay = job.hourly_pay,
                reputation_per_second = job.company_reputation_per_second,
                charisma_experience_per_second = job.charisma_experience_per_second,
                "Job available"
            );
        }
    }

    let ticker = tokio::spawn(async move {
        let mut clock = Clock::default();
        let mut interval = time::interval(Duration::from_secs_f64(1.0 / 60.0));

        loop {
            interval.tick().await;
            clock.tick();
            println!("game time {clock}");
            if clock.minute() == 1 {
                break;
            }
        }
    });

    ticker.await.expect("ticker task failed");
    info!("Game Ended");
}
