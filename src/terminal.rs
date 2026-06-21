use eframe::egui;

use crate::{DreamstackApp, HackExecution, Screen};

impl DreamstackApp {
    pub(crate) fn show_terminal(&mut self, ui: &mut egui::Ui) {
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

                if let Some(hack_execution) = &self.hack_execution {
                    let progress = hack_execution.progress();
                    let progress_text = hack_execution.progress_text();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add(egui::ProgressBar::new(progress).text(progress_text));
                        if ui.button("Skip hack").clicked() {
                            self.skip_hack_execution();
                        }
                    });
                }

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

    pub(crate) fn open_terminal(&mut self) {
        if self.has_hacking_intro()
            && self.clock.elapsed_seconds() >= self.tutorial.duration_seconds
        {
            self.terminal_input.clear();
            self.terminal_lines = vec![
                "connected to home server: dreamstack".to_string(),
                "Run `netscan` to discover nearby servers.".to_string(),
            ];
            self.terminal_scanned = false;
            self.terminal_server_scanned = false;
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

        if self.hack_execution.is_some() {
            self.terminal_lines
                .push("command blocked: hack already running".to_string());
            self.terminal_input.clear();
            self.write_autosave();
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

        self.write_autosave();
    }

    fn run_netscan(&mut self) {
        let server_lines: Vec<String> = self
            .tutorial
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
        self.terminal_server_scanned = false;
    }

    fn connect_server(&mut self, hostname: &str) {
        if !self.terminal_scanned {
            self.terminal_lines
                .push("run `netscan` before connecting".to_string());
            return;
        }

        let Some(server) = self
            .tutorial
            .servers
            .iter()
            .find(|server| server.name == hostname)
        else {
            self.terminal_lines
                .push(format!("connect failed: unknown host {hostname}"));
            return;
        };

        self.connected_server = Some(server.name.clone());
        self.terminal_server_scanned = false;
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
            .tutorial
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
        self.terminal_server_scanned = true;
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
            .tutorial
            .servers
            .iter()
            .find(|server| server.name == hostname)
        else {
            self.terminal_lines
                .push(format!("hack failed: lost connection to {hostname}"));
            return;
        };

        let hack_duration_seconds = server.hack_duration_seconds();
        self.hack_execution = Some(HackExecution {
            hostname: hostname.to_string(),
            elapsed_seconds: 0,
            duration_seconds: hack_duration_seconds,
        });
        self.terminal_lines.push(format!(
            "hack started: execution time {} in-game hours",
            hack_duration_seconds / 3_600
        ));
        self.save_status = format!("Hacking {hostname}...");
    }

    pub(crate) fn advance_hack_execution(&mut self, now: crate::FrameTime) {
        let elapsed_real_seconds = self
            .last_frame_at
            .replace(now)
            .map_or(0.0, |last_frame_at| {
                crate::elapsed_frame_seconds(now, last_frame_at)
            });
        self.real_seconds_since_autosave += elapsed_real_seconds;
        self.game_seconds_buffer += elapsed_real_seconds * crate::GAME_SECONDS_PER_REAL_SECOND;

        let Some(hack_execution) = self.hack_execution.as_mut() else {
            return;
        };

        let mut game_seconds = self.game_seconds_buffer.floor() as u64;
        if game_seconds == 0 {
            return;
        }

        let remaining_seconds = hack_execution.duration_seconds - hack_execution.elapsed_seconds;
        game_seconds = game_seconds.min(remaining_seconds);
        self.game_seconds_buffer -= game_seconds as f64;
        hack_execution.elapsed_seconds += game_seconds;
        let hack_completed = hack_execution.elapsed_seconds >= hack_execution.duration_seconds;
        self.clock.advance_by(game_seconds);

        if self.real_seconds_since_autosave >= crate::AUTOSAVE_INTERVAL_SECONDS {
            self.real_seconds_since_autosave = 0.0;
            self.write_autosave();
        }

        if hack_completed {
            self.complete_hack();
        }
    }

    fn complete_hack(&mut self) {
        let hack_execution = self
            .hack_execution
            .take()
            .expect("hack execution should be active");
        let hostname = hack_execution.hostname;
        let Some(server) = self
            .tutorial
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
        if self.tutorial.number == 1 {
            self.save_status = format!("Hacked {hostname} and completed tutorial 1.");
            self.screen = Screen::Complete;
            self.write_autosave();
            return;
        }
        if self.tutorial.number == 2 {
            self.editor_unlocked = true;
            self.terminal_lines
                .push("editor unlocked; open Hacking > Editor in the sidebar".to_string());
        }
        self.save_status = format!("Hacked {hostname} and gained {hack_experience:.3} hack exp.");
        self.write_autosave();
    }

    fn skip_hack_execution(&mut self) {
        let Some(hack_execution) = self.hack_execution.as_mut() else {
            return;
        };

        let remaining_seconds = hack_execution.duration_seconds - hack_execution.elapsed_seconds;
        hack_execution.elapsed_seconds = hack_execution.duration_seconds;
        self.clock.advance_by(remaining_seconds);
        self.complete_hack();
    }
}

impl HackExecution {
    fn progress(&self) -> f32 {
        self.elapsed_seconds as f32 / self.duration_seconds as f32
    }

    fn progress_text(&self) -> String {
        format!(
            "hacking {}: {} / {}",
            self.hostname,
            format_duration(self.elapsed_seconds),
            format_duration(self.duration_seconds)
        )
    }
}

fn format_duration(seconds: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3_600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{DreamstackApp, HackExecution, Screen, game::tutorial_1};

    fn app_ready_to_hack() -> DreamstackApp {
        let mut app = DreamstackApp {
            tutorial: tutorial_1(),
            screen: Screen::Terminal,
            terminal_scanned: true,
            terminal_server_scanned: true,
            connected_server: Some("server0".to_string()),
            root_server: Some("server0".to_string()),
            backdoor_server: Some("server0".to_string()),
            ..DreamstackApp::default()
        };
        app.clock.advance_by(app.tutorial.duration_seconds);
        app
    }

    #[test]
    fn successful_hack_starts_execution_without_completing_immediately() {
        let mut app = app_ready_to_hack();

        app.hack_connected_server();

        assert_eq!(app.clock.elapsed_seconds(), 28_800);
        assert_eq!(app.player.hack_experience(), 0.0);
        assert_eq!(app.hack_execution.unwrap().duration_seconds, 7_200);
    }

    #[test]
    fn successful_hack_completes_after_execution_time_passes() {
        let mut app = app_ready_to_hack();
        app.hack_connected_server();
        let now = Instant::now();
        app.last_frame_at = Some(now - Duration::from_secs(120));

        app.advance_hack_execution(now);

        assert_eq!(app.clock.elapsed_seconds(), 36_000);
        assert_eq!(app.player.hack_experience(), 25.0);
        assert!(app.hack_execution.is_none());
    }

    #[test]
    fn skip_hack_completes_active_hack() {
        let mut app = app_ready_to_hack();
        app.hack_connected_server();

        app.skip_hack_execution();

        assert_eq!(app.clock.elapsed_seconds(), 36_000);
        assert_eq!(app.player.hack_experience(), 25.0);
        assert!(app.hack_execution.is_none());
    }

    #[test]
    fn hack_progress_text_shows_elapsed_and_total_time() {
        let hack_execution = HackExecution {
            hostname: "server0".to_string(),
            elapsed_seconds: 3_600,
            duration_seconds: 7_200,
        };

        assert_eq!(hack_execution.progress(), 0.5);
        assert_eq!(
            hack_execution.progress_text(),
            "hacking server0: 01:00:00 / 02:00:00"
        );
    }
}
