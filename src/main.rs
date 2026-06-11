use std::{
    io::{self, Write},
    sync::mpsc::{self, TryRecvError},
};

use tokio::time::{self, Duration};

mod game;
mod player;
mod save;

use game::{Clock, level_0};
use player::Player;
use save::{SaveFile, SavedActiveJob, write_autosave};
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
    let mut player = Player::default();
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
    println!("Once your 8-hour shift starts, you cannot do anything else until it is over.");

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
    write_level_autosave(&player, level.number, &clock, &employer.name, &job.name)?;

    let mut interval = time::interval(Duration::from_secs_f64(1.0 / 60.0));
    let mut next_autosave_at = time::Instant::now() + Duration::from_secs(5 * 60);
    let skip_prompt_at = time::Instant::now() + Duration::from_secs(10);
    let mut skip_prompt_shown = false;
    let mut skip_answer = None;

    while clock.elapsed_seconds() < level.duration_seconds {
        interval.tick().await;
        clock.tick();
        player.earn_money(job.pay_for_seconds(1));
        player.gain_company_reputation(&employer.name, job.company_reputation_for_seconds(1));
        player.gain_charisma_experience(job.charisma_experience_for_seconds(1));

        if clock.second() == 0 {
            println!(
                "game time {clock} | money {:.3} | company rep {:.3} | favor {:.3} | charisma exp {:.3}",
                player.money(),
                player.company_reputation(&employer.name),
                player.company_favor(&employer.name),
                player.charisma_experience()
            );
        }

        if time::Instant::now() >= next_autosave_at {
            write_level_autosave(&player, level.number, &clock, &employer.name, &job.name)?;
            next_autosave_at += Duration::from_secs(5 * 60);
        }

        if !skip_prompt_shown && time::Instant::now() >= skip_prompt_at {
            skip_prompt_shown = true;
            let (sender, receiver) = mpsc::channel();
            skip_answer = Some(receiver);

            tokio::task::spawn_blocking(move || {
                let result = prompt_to_skip_level();
                let _ = sender.send(result);
            });
        }

        if let Some(receiver) = &skip_answer {
            match receiver.try_recv() {
                Ok(true) => {
                    let remaining_seconds = level.duration_seconds - clock.elapsed_seconds();
                    player.earn_money(job.pay_for_seconds(remaining_seconds));
                    player.gain_company_reputation(
                        &employer.name,
                        job.company_reputation_for_seconds(remaining_seconds),
                    );
                    player.gain_charisma_experience(
                        job.charisma_experience_for_seconds(remaining_seconds),
                    );
                    clock.advance_by(remaining_seconds);
                    write_level_autosave(&player, level.number, &clock, &employer.name, &job.name)?;
                    println!("Skipping the rest of level {}.", level.number);
                    break;
                }
                Ok(false) => {
                    println!("Continuing level {}.", level.number);
                    skip_answer = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    skip_answer = None;
                }
            }
        }
    }

    write_level_autosave(&player, level.number, &clock, &employer.name, &job.name)?;

    println!("Level {} complete.", level.number);
    println!("Money: {:.3}", player.money());
    println!(
        "{} reputation: {:.3}",
        employer.name,
        player.company_reputation(&employer.name)
    );
    println!(
        "{} favor: {:.3}",
        employer.name,
        player.company_favor(&employer.name)
    );
    println!("Charisma exp: {:.3}", player.charisma_experience());

    if prompt_to_convert_company_reputation()? {
        player.shift_exchange_marker(-1.0);
        println!("Company reputation will carry forward as favor.");
    } else {
        player.shift_exchange_marker(1.0);
        player.clear_company_standings();
        println!("Company reputation will not carry forward.");
    }

    write_level_autosave(&player, level.number, &clock, &employer.name, &job.name)?;
    info!("Game Ended");

    Ok(())
}

fn prompt_to_skip_level() -> bool {
    print!("\nSkip the rest of this level? [y/N]: ");
    let _ = io::stdout().flush();

    let mut answer = String::new();
    match io::stdin().read_line(&mut answer) {
        Ok(_) => answer.trim().eq_ignore_ascii_case("y"),
        Err(error) => {
            eprintln!("Could not read skip answer: {error}");
            false
        }
    }
}

fn prompt_to_convert_company_reputation() -> io::Result<bool> {
    print!("Convert all company reputation into favor for the next part of the game? [y/N]: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    Ok(answer.trim().eq_ignore_ascii_case("y"))
}

fn write_level_autosave(
    player: &Player,
    active_level: u8,
    clock: &Clock,
    employer_name: &str,
    job_name: &str,
) -> io::Result<()> {
    let save_file = SaveFile::new(
        player.to_save(),
        active_level,
        clock.elapsed_seconds(),
        Some(SavedActiveJob::new(employer_name, job_name)),
    );

    write_autosave(&save_file)
}
