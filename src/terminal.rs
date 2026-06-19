use eframe::egui;

use crate::{DreamstackApp, Screen};

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
}
