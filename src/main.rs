use std::{io, time::Instant};

use eframe::egui;

mod game;
mod log;
mod player;
mod save;

use game::{Clock, Level, company_reputation_rate_multiplier, level_0, level_1};
use player::Player;
use save::{SaveFile, SavedActiveJob, write_autosave};
use tracing::info;

const AUTOSAVE_INTERVAL_SECONDS: f64 = 5.0 * 60.0;
const GAME_SECONDS_PER_REAL_SECOND: f64 = 60.0;

fn main() -> eframe::Result {
    log::init_tracing();
    info!("Game Started");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([880.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dreamstack",
        options,
        Box::new(|_creation_context| Ok(Box::new(DreamstackApp::default()))),
    )
}

struct DreamstackApp {
    player: Player,
    level: Level,
    clock: Clock,
    screen: Screen,
    sidebar_open: bool,
    terminal_input: String,
    terminal_lines: Vec<String>,
    terminal_scanned: bool,
    company_reputation_rate_favor: f64,
    last_frame_at: Option<Instant>,
    game_seconds_buffer: f64,
    real_seconds_since_autosave: f64,
    save_status: String,
}

impl Default for DreamstackApp {
    fn default() -> Self {
        Self {
            player: Player::default(),
            level: level_0(),
            clock: Clock::default(),
            screen: Screen::default(),
            sidebar_open: true,
            terminal_input: String::new(),
            terminal_lines: Vec::new(),
            terminal_scanned: false,
            company_reputation_rate_favor: 0.0,
            last_frame_at: None,
            game_seconds_buffer: 0.0,
            real_seconds_since_autosave: 0.0,
            save_status: String::new(),
        }
    }
}

#[derive(Default, PartialEq)]
enum Screen {
    #[default]
    Intro,
    Working,
    TerminalPrompt,
    Terminal,
    Complete,
    Finished,
}

impl eframe::App for DreamstackApp {
    fn logic(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if self.screen == Screen::Working {
            self.advance_working_time();
            context.request_repaint();
        } else {
            self.last_frame_at = Some(Instant::now());
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("sidebar")
            .resizable(false)
            .exact_size(if self.sidebar_open { 160.0 } else { 36.0 })
            .show_inside(ui, |ui| {
                let toggle_label = if self.sidebar_open { "<" } else { ">" };
                if ui.button(toggle_label).clicked() {
                    self.sidebar_open = !self.sidebar_open;
                }

                if self.sidebar_open {
                    ui.add_space(12.0);
                    egui::CollapsingHeader::new("Hacking").show(ui, |ui| {
                        if ui
                            .add_enabled(self.level.number != 0, egui::Button::new("Terminal"))
                            .clicked()
                        {
                            self.open_terminal();
                        }
                    });
                }
            });

        egui::Panel::right("player_stats_sidebar")
            .resizable(false)
            .exact_size(220.0)
            .show_inside(ui, |ui| {
                ui.heading("Player");
                ui.add_space(8.0);
                self.show_stats(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Dreamstack");
            ui.add_space(8.0);

            match self.screen {
                Screen::Intro => self.show_intro(ui),
                Screen::Working => self.show_working(ui),
                Screen::TerminalPrompt => self.show_terminal_prompt(ui),
                Screen::Terminal => self.show_terminal(ui),
                Screen::Complete => self.show_complete(ui),
                Screen::Finished => self.show_finished(ui),
            }

            if !self.save_status.is_empty() {
                ui.add_space(16.0);
                ui.label(&self.save_status);
            }
        });
    }
}

impl DreamstackApp {
    fn employer(&self) -> &game::Employer {
        self.level
            .employers
            .first()
            .expect("level should have an employer")
    }

    fn job(&self) -> &game::Job {
        self.employer()
            .jobs
            .first()
            .expect("level should have a job")
    }

    fn show_intro(&mut self, ui: &mut egui::Ui) {
        let employer = self.employer();
        let job = self.job();

        ui.label(format!(
            "Level {}: {}",
            self.level.number,
            self.level_title()
        ));
        ui.label(format!(
            "Complete one {}-hour work shift.",
            self.level.duration_seconds / 3_600
        ));
        ui.add_space(12.0);
        ui.label(format!(
            "{} is hiring for one role: {}.",
            employer.name, job.name
        ));
        ui.label(format!("Pay: {:.3}/hr", job.hourly_pay));
        ui.label(format!(
            "While working, you gain {:.3} company reputation and {:.3} charisma exp per in-game second.",
            job.company_reputation_per_second, job.charisma_experience_per_second
        ));
        ui.label("Once your 8-hour shift starts, you cannot do anything else until it is over.");
        if self.has_hacking_intro() {
            ui.label("After the shift, you will unlock your first hacking target.");
        }
        ui.add_space(20.0);

        if ui
            .button(format!("Take the {} job at {}", job.name, employer.name))
            .clicked()
        {
            self.start_job();
        }
    }

    fn show_working(&mut self, ui: &mut egui::Ui) {
        let progress = self.clock.elapsed_seconds() as f32 / self.level.duration_seconds as f32;

        ui.label(format!(
            "Working as {} at {}",
            self.job().name,
            self.employer().name
        ));
        ui.add(egui::ProgressBar::new(progress).text(format!(
            "game time {} / {}:00:00",
            self.clock,
            self.level.duration_seconds / 3_600
        )));
        ui.add_space(20.0);

        if ui.button("Skip the rest of this level").clicked() {
            self.skip_level();
        }
    }

    fn show_complete(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Level {} complete.", self.level.number));
        ui.add_space(20.0);
        ui.label("Convert all company reputation into favor for the next part of the game?");

        ui.horizontal(|ui| {
            if ui.button("Convert to favor").clicked() {
                self.player.shift_exchange_marker(-1.0);
                self.advance_after_reputation_choice(
                    "Company reputation will carry forward as favor.",
                );
            }

            if ui.button("Do not convert").clicked() {
                self.player.shift_exchange_marker(1.0);
                self.player.clear_company_standings();
                self.advance_after_reputation_choice("Company reputation will not carry forward.");
            }
        });
    }

    fn show_terminal_prompt(&self, ui: &mut egui::Ui) {
        ui.label(format!("Level {} shift complete.", self.level.number));
        ui.add_space(12.0);
        ui.label("Open the Hacking group in the sidebar and press Terminal to continue.");
    }

    fn show_terminal(&mut self, ui: &mut egui::Ui) {
        ui.label("Terminal");
        ui.add_space(8.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.terminal_lines {
                        ui.monospace(line);
                    }
                });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.monospace(">");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.terminal_input)
                    .desired_width(320.0)
                    .hint_text("netscan"),
            );
            let pressed_enter =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

            if ui.button("Run").clicked() || pressed_enter {
                self.run_terminal_command();
            }
        });
    }

    fn show_finished(&self, ui: &mut egui::Ui) {
        ui.label("Run complete.");
    }

    fn show_stats(&self, ui: &mut egui::Ui) {
        egui::Grid::new("player_stats")
            .num_columns(2)
            .spacing([24.0, 8.0])
            .show(ui, |ui| {
                ui.label("Level");
                ui.label(self.level.number.to_string());
                ui.end_row();

                ui.label("Money");
                ui.label(format!("{:.3}", self.player.money()));
                ui.end_row();

                ui.label("Charisma skill");
                ui.label(self.player.charisma_skill().to_string());
                ui.end_row();

                ui.label("Charisma exp");
                ui.label(format!("{:.3}", self.player.charisma_experience()));
                ui.end_row();

                ui.label("Hack skill");
                ui.label(self.player.hack_skill().to_string());
                ui.end_row();

                ui.label("Hack exp");
                ui.label(format!("{:.3}", self.player.hack_experience()));
                ui.end_row();
            });
    }

    fn start_job(&mut self) {
        self.screen = Screen::Working;
        self.last_frame_at = Some(Instant::now());
        self.real_seconds_since_autosave = 0.0;
        self.save_status = format!(
            "You took the {} job. Starting level {}.",
            self.job().name,
            self.level.number
        );
        self.write_autosave();
    }

    fn advance_working_time(&mut self) {
        let now = Instant::now();
        let elapsed_real_seconds = self
            .last_frame_at
            .replace(now)
            .map_or(0.0, |last_frame_at| {
                now.duration_since(last_frame_at).as_secs_f64()
            });

        self.real_seconds_since_autosave += elapsed_real_seconds;
        self.game_seconds_buffer += elapsed_real_seconds * GAME_SECONDS_PER_REAL_SECOND;

        let mut game_seconds = self.game_seconds_buffer.floor() as u64;
        if game_seconds == 0 {
            return;
        }

        self.game_seconds_buffer -= game_seconds as f64;
        let remaining_seconds = self.level.duration_seconds - self.clock.elapsed_seconds();
        game_seconds = game_seconds.min(remaining_seconds);
        self.apply_work_rewards(game_seconds);
        self.clock.advance_by(game_seconds);

        if self.real_seconds_since_autosave >= AUTOSAVE_INTERVAL_SECONDS {
            self.real_seconds_since_autosave = 0.0;
            self.write_autosave();
        }

        if self.clock.elapsed_seconds() >= self.level.duration_seconds {
            self.finish_level();
        }
    }

    fn skip_level(&mut self) {
        let remaining_seconds = self.level.duration_seconds - self.clock.elapsed_seconds();
        self.apply_work_rewards(remaining_seconds);
        self.clock.advance_by(remaining_seconds);
        self.save_status = format!("Skipped the rest of level {}.", self.level.number);
        self.finish_level();
    }

    fn finish_level(&mut self) {
        self.screen = if self.has_hacking_intro() {
            Screen::TerminalPrompt
        } else {
            Screen::Complete
        };
        self.write_autosave();
    }

    fn advance_after_reputation_choice(&mut self, save_status: &str) {
        self.save_status = save_status.to_string();

        if self.level.number == 0 {
            self.start_level_1();
        } else {
            self.write_autosave();
            self.screen = Screen::Finished;
        }
    }

    fn start_level_1(&mut self) {
        let employer_name = self.employer().name.clone();
        self.company_reputation_rate_favor = self.player.company_favor(&employer_name);
        self.player.reset_money();
        self.player.clear_skill_experience();
        self.player.clear_company_standings();

        self.level = level_1();
        self.clock = Clock::default();
        self.screen = Screen::Intro;
        self.last_frame_at = None;
        self.game_seconds_buffer = 0.0;
        self.real_seconds_since_autosave = 0.0;
        self.save_status.push_str(" Starting level 1.");
        self.write_autosave();
    }

    fn level_title(&self) -> &'static str {
        if self.has_hacking_intro() {
            "Hacking System"
        } else {
            "Job System"
        }
    }

    fn has_hacking_intro(&self) -> bool {
        !self.level.servers.is_empty()
    }

    fn open_terminal(&mut self) {
        if self.has_hacking_intro() && self.clock.elapsed_seconds() >= self.level.duration_seconds {
            self.terminal_input.clear();
            self.terminal_lines = vec![
                "Dreamstack terminal online.".to_string(),
                "Run `netscan` to discover nearby servers.".to_string(),
            ];
            self.terminal_scanned = false;
            self.save_status = "Terminal opened.".to_string();
            self.screen = Screen::Terminal;
        } else {
            self.save_status = "Finish your shift before using Terminal.".to_string();
        }
    }

    fn run_terminal_command(&mut self) {
        let command = self.terminal_input.trim().to_string();
        if command.is_empty() {
            return;
        }

        self.terminal_lines.push(format!("> {command}"));
        self.terminal_input.clear();

        match command.as_str() {
            "netscan" => self.run_netscan(),
            _ => self
                .terminal_lines
                .push(format!("unknown command: {command}")),
        }
    }

    fn run_netscan(&mut self) {
        let server_lines: Vec<String> = self
            .level
            .servers
            .iter()
            .map(|server| {
                format!(
                    "{} | skill {} | security {:.1} | money {:.3}",
                    server.name,
                    server.hack_skill_needed,
                    server.min_security,
                    server.max_money()
                )
            })
            .collect();

        if server_lines.is_empty() {
            self.terminal_lines.push("no servers found".to_string());
            return;
        }

        self.terminal_lines.push("servers found:".to_string());
        self.terminal_lines.extend(server_lines);
        self.terminal_scanned = true;
    }

    fn apply_work_rewards(&mut self, seconds: u64) {
        let employer_name = self.employer().name.clone();
        let pay = self.job().pay_for_seconds(seconds);
        let company_reputation = self.job().company_reputation_for_seconds(seconds)
            * company_reputation_rate_multiplier(self.company_reputation_rate_favor);
        let charisma_experience = self.job().charisma_experience_for_seconds(seconds);

        self.player.earn_money(pay);
        self.player
            .gain_company_reputation(&employer_name, company_reputation);
        self.player.gain_charisma_experience(charisma_experience);
    }

    fn write_autosave(&mut self) {
        let employer_name = self.employer().name.clone();
        let job_name = self.job().name.clone();

        match write_level_autosave(
            &self.player,
            self.level.number,
            &self.clock,
            &employer_name,
            &job_name,
        ) {
            Ok(()) => {
                if self.save_status.is_empty() {
                    self.save_status = "Autosaved.".to_string();
                }
            }
            Err(error) => {
                self.save_status = format!("Autosave failed: {error}");
            }
        }
    }
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
