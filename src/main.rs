use std::io;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use eframe::egui;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, prelude::*};

mod game;
#[cfg(not(target_arch = "wasm32"))]
mod log;
mod player;
mod save;

use game::{Clock, Level, company_reputation_rate_multiplier, level_0, level_1};
use player::Player;
use save::{SaveFile, SavedActiveJob, write_autosave};
#[cfg(not(target_arch = "wasm32"))]
use tracing::info;

const AUTOSAVE_INTERVAL_SECONDS: f64 = 5.0 * 60.0;
const GAME_SECONDS_PER_REAL_SECOND: f64 = 60.0;

#[cfg(not(target_arch = "wasm32"))]
type FrameTime = Instant;
#[cfg(target_arch = "wasm32")]
type FrameTime = f64;

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("dreamstack_canvas"))
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("missing dreamstack_canvas element"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|_creation_context| Ok(Box::new(DreamstackApp::default()))),
        )
        .await
}

#[cfg(target_arch = "wasm32")]
fn main() {}

struct DreamstackApp {
    player: Player,
    level: Level,
    clock: Clock,
    screen: Screen,
    sidebar_open: bool,
    player_sidebar_open: bool,
    terminal_input: String,
    terminal_lines: Vec<String>,
    terminal_scanned: bool,
    connected_server: Option<String>,
    root_server: Option<String>,
    backdoor_server: Option<String>,
    company_reputation_rate_favor: f64,
    last_frame_at: Option<FrameTime>,
    game_seconds_buffer: f64,
    real_seconds_since_autosave: f64,
    save_status: String,
}

#[cfg(not(target_arch = "wasm32"))]
fn frame_time_now(_context: &egui::Context) -> FrameTime {
    Instant::now()
}

#[cfg(target_arch = "wasm32")]
fn frame_time_now(context: &egui::Context) -> FrameTime {
    context.input(|input| input.time)
}

#[cfg(not(target_arch = "wasm32"))]
fn elapsed_frame_seconds(now: FrameTime, last_frame_at: FrameTime) -> f64 {
    now.duration_since(last_frame_at).as_secs_f64()
}

#[cfg(target_arch = "wasm32")]
fn elapsed_frame_seconds(now: FrameTime, last_frame_at: FrameTime) -> f64 {
    (now - last_frame_at).max(0.0)
}

impl Default for DreamstackApp {
    fn default() -> Self {
        Self {
            player: Player::default(),
            level: level_0(),
            clock: Clock::default(),
            screen: Screen::default(),
            sidebar_open: true,
            player_sidebar_open: true,
            terminal_input: String::new(),
            terminal_lines: Vec::new(),
            terminal_scanned: false,
            connected_server: None,
            root_server: None,
            backdoor_server: None,
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
            self.advance_working_time(frame_time_now(context));
            context.request_repaint();
        } else {
            self.last_frame_at = Some(frame_time_now(context));
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
            .exact_size(if self.player_sidebar_open {
                220.0
            } else {
                36.0
            })
            .show_inside(ui, |ui| {
                let toggle_label = if self.player_sidebar_open { ">" } else { "<" };
                if ui.button(toggle_label).clicked() {
                    self.player_sidebar_open = !self.player_sidebar_open;
                }

                if self.player_sidebar_open {
                    ui.add_space(12.0);
                    ui.heading("Player");
                    ui.add_space(8.0);
                    self.show_stats(ui);
                }
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
        let terminal_text = egui::Color32::from_rgb(125, 255, 162);
        let terminal_dim = egui::Color32::from_rgb(66, 132, 86);
        let terminal_fill = egui::Color32::from_rgb(5, 10, 7);

        egui::Frame::group(ui.style())
            .fill(terminal_fill)
            .stroke(egui::Stroke::new(1.0, terminal_dim))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(terminal_text);
                ui.monospace("home server terminal");
                ui.separator();

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height((ui.available_height() - 44.0).max(240.0))
                    .show(ui, |ui| {
                        for line in &self.terminal_lines {
                            ui.monospace(line);
                        }
                    });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.monospace(self.terminal_prompt());
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_input)
                            .desired_width(f32::INFINITY)
                            .hint_text("netscan")
                            .text_color(terminal_text)
                            .frame(egui::Frame::NONE),
                    );
                    response.request_focus();

                    let pressed_enter = response.has_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter));

                    if pressed_enter {
                        self.run_terminal_command();
                    }
                });
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
        self.last_frame_at = None;
        self.real_seconds_since_autosave = 0.0;
        self.save_status = format!(
            "You took the {} job. Starting level {}.",
            self.job().name,
            self.level.number
        );
        self.write_autosave();
    }

    fn advance_working_time(&mut self, now: FrameTime) {
        let elapsed_real_seconds = self
            .last_frame_at
            .replace(now)
            .map_or(0.0, |last_frame_at| {
                elapsed_frame_seconds(now, last_frame_at)
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
                "connected to home server: dreamstack".to_string(),
                "Run `netscan` to discover nearby servers.".to_string(),
            ];
            self.terminal_scanned = false;
            self.connected_server = None;
            self.root_server = None;
            self.backdoor_server = None;
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

        self.terminal_lines
            .push(format!("{} {command}", self.terminal_prompt()));
        self.terminal_input.clear();

        if command == "netscan" {
            self.run_netscan();
        } else if let Some(hostname) = command.strip_prefix("connect ") {
            self.connect_server(hostname);
        } else if command == "scan" {
            self.scan_connected_server();
        } else if command == "nuke" {
            self.nuke_connected_server();
        } else if command == "npm i -g backdoor" {
            self.install_backdoor();
        } else if command == "hack" {
            self.hack_connected_server();
        } else if command == "home" {
            self.return_home();
        } else {
            self.terminal_lines
                .push(format!("unknown command: {command}"));
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
        self.terminal_lines
            .push("Run `connect <hostname>` to connect to a server.".to_string());
        self.terminal_scanned = true;
    }

    fn terminal_prompt(&self) -> String {
        if let Some(hostname) = self.connected_server.as_deref() {
            let username = if self.root_server.as_deref() == Some(hostname) {
                "root"
            } else {
                "user"
            };

            format!("{username}@{hostname}:~$")
        } else {
            "home@dreamstack:~$".to_string()
        }
    }

    fn return_home(&mut self) {
        let Some(hostname) = self.connected_server.take() else {
            self.terminal_lines
                .push("already connected to home server".to_string());
            return;
        };

        self.terminal_lines
            .push(format!("disconnected from {hostname}; returned home"));
    }

    fn connect_server(&mut self, hostname: &str) {
        if !self.terminal_scanned {
            self.terminal_lines
                .push("run `netscan` before connecting".to_string());
            return;
        }

        let Some(server) = self
            .level
            .servers
            .iter()
            .find(|server| server.name == hostname)
        else {
            self.terminal_lines
                .push(format!("connect failed: unknown host {hostname}"));
            return;
        };

        self.connected_server = Some(server.name.clone());
        self.terminal_lines
            .push(format!("connected to {}", server.name));
        self.terminal_lines
            .push("Run `scan` to inspect the connected server.".to_string());
    }

    fn scan_connected_server(&mut self) {
        let Some(hostname) = self.connected_server.as_deref() else {
            self.terminal_lines
                .push("connect to a server before scanning".to_string());
            return;
        };

        let Some(server) = self
            .level
            .servers
            .iter()
            .find(|server| server.name == hostname)
        else {
            self.terminal_lines
                .push(format!("scan failed: lost connection to {hostname}"));
            return;
        };

        self.terminal_lines
            .push(format!("scan report: {}", server.name));
        self.terminal_lines
            .push(format!("required hack skill: {}", server.hack_skill_needed));
        self.terminal_lines
            .push(format!("minimum security: {:.1}", server.min_security));
        self.terminal_lines
            .push(format!("maximum money: {:.3}", server.max_money()));
        self.terminal_lines.push(
            "Run `nuke`; if successful, it will give you root access to the server.".to_string(),
        );
    }

    fn nuke_connected_server(&mut self) {
        let Some(hostname) = self.connected_server.as_deref() else {
            self.terminal_lines
                .push("connect to a server before running nuke".to_string());
            return;
        };

        self.root_server = Some(hostname.to_string());
        self.terminal_lines.push(format!(
            "nuke successful: root access granted on {hostname}"
        ));
        self.terminal_lines
            .push("Run `npm i -g backdoor` to install a backdoor.".to_string());
    }

    fn install_backdoor(&mut self) {
        let Some(hostname) = self.connected_server.as_deref() else {
            self.terminal_lines
                .push("connect to a server before installing a backdoor".to_string());
            return;
        };

        if self.root_server.as_deref() != Some(hostname) {
            self.terminal_lines
                .push("root access required before installing a backdoor".to_string());
            return;
        }

        self.backdoor_server = Some(hostname.to_string());
        self.terminal_lines
            .push(format!("backdoor installed on {hostname}"));
        self.terminal_lines
            .push("Run `hack` to drain the server.".to_string());
    }

    fn hack_connected_server(&mut self) {
        let Some(hostname) = self.connected_server.as_deref() else {
            self.terminal_lines
                .push("connect to a server before hacking".to_string());
            return;
        };

        if self.backdoor_server.as_deref() != Some(hostname) {
            self.terminal_lines
                .push("install a backdoor before hacking".to_string());
            return;
        }

        let Some(server) = self
            .level
            .servers
            .iter()
            .find(|server| server.name == hostname)
        else {
            self.terminal_lines
                .push(format!("hack failed: lost connection to {hostname}"));
            return;
        };

        let hack_experience = server.hack_experience_reward();
        self.player.gain_hack_experience(hack_experience);
        self.terminal_lines.push(format!(
            "hack complete: gained {hack_experience:.3} hack exp"
        ));
        self.save_status = format!("Hacked {hostname} and gained {hack_experience:.3} hack exp.");
        self.write_autosave();
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
