use std::io;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use eframe::egui;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, prelude::*};

mod ds;
mod editor;
mod game;
#[cfg(not(target_arch = "wasm32"))]
mod log;
mod player;
mod save;
mod terminal;

use game::{
    Clock, Tutorial, company_reputation_rate_multiplier, tutorial_0, tutorial_1, tutorial_2,
};
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

pub(crate) struct DreamstackApp {
    pub(crate) player: Player,
    pub(crate) tutorial: Tutorial,
    pub(crate) clock: Clock,
    pub(crate) screen: Screen,
    sidebar_open: bool,
    player_sidebar_open: bool,
    pub(crate) terminal_input: String,
    pub(crate) terminal_lines: Vec<String>,
    pub(crate) terminal_scanned: bool,
    pub(crate) connected_server: Option<String>,
    pub(crate) root_server: Option<String>,
    pub(crate) backdoor_server: Option<String>,
    pub(crate) editor_unlocked: bool,
    pub(crate) editor_text: String,
    pub(crate) editor_output: Vec<String>,
    company_reputation_rate_favor: f64,
    last_frame_at: Option<FrameTime>,
    game_seconds_buffer: f64,
    real_seconds_since_autosave: f64,
    pub(crate) save_status: String,
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
            tutorial: tutorial_0(),
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
            editor_unlocked: false,
            editor_text: "fn main(ds) {\n    ds_print(ds, \"hello from automation\");\n}\n"
                .to_string(),
            editor_output: Vec::new(),
            company_reputation_rate_favor: 0.0,
            last_frame_at: None,
            game_seconds_buffer: 0.0,
            real_seconds_since_autosave: 0.0,
            save_status: String::new(),
        }
    }
}

#[derive(Default, PartialEq)]
pub(crate) enum Screen {
    #[default]
    Intro,
    Working,
    TerminalPrompt,
    Terminal,
    Editor,
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
                            .add_enabled(self.tutorial.number != 0, egui::Button::new("Terminal"))
                            .clicked()
                        {
                            self.open_terminal();
                        }

                        if ui
                            .add_enabled(self.editor_unlocked, egui::Button::new("Editor"))
                            .clicked()
                        {
                            self.open_editor();
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
                Screen::Editor => self.show_editor(ui),
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
    pub(crate) fn employer(&self) -> &game::Employer {
        self.tutorial
            .employers
            .first()
            .expect("tutorial should have an employer")
    }

    pub(crate) fn job(&self) -> &game::Job {
        self.employer()
            .jobs
            .first()
            .expect("tutorial should have a job")
    }

    fn show_intro(&mut self, ui: &mut egui::Ui) {
        let employer = self.employer();
        let job = self.job();

        ui.label(format!(
            "Tutorial {}: {}",
            self.tutorial.number,
            self.tutorial_title()
        ));
        ui.label(format!(
            "Complete one {}-hour work shift.",
            self.tutorial.duration_seconds / 3_600
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
        let progress = self.clock.elapsed_seconds() as f32 / self.tutorial.duration_seconds as f32;

        ui.label(format!(
            "Working as {} at {}",
            self.job().name,
            self.employer().name
        ));
        ui.add(egui::ProgressBar::new(progress).text(format!(
            "game time {} / {}:00:00",
            self.clock,
            self.tutorial.duration_seconds / 3_600
        )));
        ui.add_space(20.0);

        if ui.button("Skip the rest of this shift").clicked() {
            self.skip_tutorial();
        }
    }

    fn show_complete(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Tutorial {} complete.", self.tutorial.number));
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
        ui.label(format!("Tutorial {} shift complete.", self.tutorial.number));
        ui.add_space(12.0);
        ui.label("Open the Hacking group in the sidebar and press Terminal to continue.");
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
            "You took the {} job. Starting tutorial {}.",
            self.job().name,
            self.tutorial.number
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
        let remaining_seconds = self.tutorial.duration_seconds - self.clock.elapsed_seconds();
        game_seconds = game_seconds.min(remaining_seconds);
        self.apply_work_rewards(game_seconds);
        self.clock.advance_by(game_seconds);

        if self.real_seconds_since_autosave >= AUTOSAVE_INTERVAL_SECONDS {
            self.real_seconds_since_autosave = 0.0;
            self.write_autosave();
        }

        if self.clock.elapsed_seconds() >= self.tutorial.duration_seconds {
            self.finish_tutorial();
        }
    }

    fn skip_tutorial(&mut self) {
        let remaining_seconds = self.tutorial.duration_seconds - self.clock.elapsed_seconds();
        self.apply_work_rewards(remaining_seconds);
        self.clock.advance_by(remaining_seconds);
        self.save_status = format!("Skipped the rest of tutorial {}.", self.tutorial.number);
        self.finish_tutorial();
    }

    fn finish_tutorial(&mut self) {
        self.screen = if self.has_hacking_intro() {
            Screen::TerminalPrompt
        } else {
            Screen::Complete
        };
        self.write_autosave();
    }

    fn advance_after_reputation_choice(&mut self, save_status: &str) {
        self.save_status = save_status.to_string();

        match self.tutorial.number {
            0 => self.start_tutorial_1(),
            1 => self.start_tutorial_2(),
            _ => {
                self.write_autosave();
                self.screen = Screen::Finished;
            }
        }
    }

    fn start_tutorial_1(&mut self) {
        self.start_next_tutorial(tutorial_1(), " Starting tutorial 1.");
    }

    fn start_tutorial_2(&mut self) {
        self.start_next_tutorial(tutorial_2(), " Starting tutorial 2.");
    }

    fn start_next_tutorial(&mut self, tutorial: Tutorial, save_status_suffix: &str) {
        let employer_name = self.employer().name.clone();
        self.company_reputation_rate_favor = self.player.company_favor(&employer_name);
        self.player.reset_money();
        self.player.clear_skill_experience();
        self.player.clear_company_standings();

        self.tutorial = tutorial;
        self.clock = Clock::default();
        self.screen = Screen::Intro;
        self.last_frame_at = None;
        self.game_seconds_buffer = 0.0;
        self.real_seconds_since_autosave = 0.0;
        self.save_status.push_str(save_status_suffix);
        self.write_autosave();
    }

    fn tutorial_title(&self) -> &'static str {
        if self.has_hacking_intro() {
            "Hacking System"
        } else {
            "Job System"
        }
    }

    pub(crate) fn has_hacking_intro(&self) -> bool {
        !self.tutorial.servers.is_empty()
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

    pub(crate) fn write_autosave(&mut self) {
        let employer_name = self.employer().name.clone();
        let job_name = self.job().name.clone();

        match write_tutorial_autosave(
            &self.player,
            self.tutorial.number,
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

fn write_tutorial_autosave(
    player: &Player,
    active_tutorial: u8,
    clock: &Clock,
    employer_name: &str,
    job_name: &str,
) -> io::Result<()> {
    let save_file = SaveFile::new(
        player.to_save(),
        active_tutorial,
        clock.elapsed_seconds(),
        Some(SavedActiveJob::new(employer_name, job_name)),
    );

    write_autosave(&save_file)
}
