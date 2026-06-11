use std::io::{self, Write};

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
    if let Err(error) = run().await {
        eprintln!("Game failed: {error}");
    }
}

async fn run() -> io::Result<()> {
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
    let employer = level
        .employers
        .first()
        .expect("level 0 should have an employer");
    let job = employer.jobs.first().expect("level 0 should have a job");

    info!(
        level = level.number,
        duration_seconds = level.duration_seconds,
        employers = level.employers.len(),
        "Level loaded"
    );

    info!(
        employer = employer.name,
        job = job.name,
        hourly_pay = job.hourly_pay,
        reputation_per_second = job.company_reputation_per_second,
        charisma_experience_per_second = job.charisma_experience_per_second,
        "Job available"
    );

    println!("Level {}: Job System", level.number);
    println!(
        "This level lasts {} in-game hours.",
        level.duration_seconds / 3_600
    );
    println!("{} is hiring for one role: {}.", employer.name, job.name);
    println!("Pay: {:.3}/hr", job.hourly_pay);
    println!(
        "While working, you gain {:.3} company reputation and {:.3} charisma exp per in-game second.",
        job.company_reputation_per_second, job.charisma_experience_per_second
    );

    print!("Take the {} job at {}? [y/N]: ", job.name, employer.name);
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    if !answer.trim().eq_ignore_ascii_case("y") {
        println!("No job taken. Level {} cannot start yet.", level.number);
        info!("Game Ended");
        return Ok(());
    }

    println!(
        "You took the {} job. Starting level {}.",
        job.name, level.number
    );

    let mut clock = Clock::default();
    let mut interval = time::interval(Duration::from_secs_f64(1.0 / 60.0));

    while clock.elapsed_seconds() < level.duration_seconds {
        interval.tick().await;
        clock.tick();

        if clock.second() == 0 {
            println!("game time {clock}");
        }
    }

    println!("Level {} complete.", level.number);
    info!("Game Ended");

    Ok(())
}
