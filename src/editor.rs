use eframe::egui;

use crate::{DreamstackApp, Screen, ds};

impl DreamstackApp {
    pub(crate) fn show_editor(&mut self, ui: &mut egui::Ui) {
        ui.label("Rhai Automation Editor");
        ui.add_space(8.0);
        ui.label("Write Rhai automation scripts here.");
        ui.add_space(8.0);

        ui.add(
            egui::TextEdit::multiline(&mut self.editor_text)
                .desired_rows(16)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace)
                .hint_text("// Rhai automation script"),
        );

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Run").clicked() {
                match ds::run_script(&self.editor_text) {
                    Ok(output) => {
                        self.editor_output = output;
                        self.save_status = "Rhai script ran.".to_string();
                    }
                    Err(error) => {
                        self.editor_output.clear();
                        self.save_status = format!("Rhai script failed: {error}");
                    }
                }
            }

            if ui.button("Clear").clicked() {
                self.editor_text.clear();
                self.save_status = "Editor cleared.".to_string();
            }

            ui.label(format!("{} chars", self.editor_text.chars().count()));
        });

        if !self.editor_output.is_empty() {
            ui.add_space(8.0);
            ui.label("Output");
            for line in &self.editor_output {
                ui.monospace(line);
            }
        }
    }

    pub(crate) fn open_editor(&mut self) {
        if self.editor_unlocked {
            self.screen = Screen::Editor;
            self.save_status = "Editor opened.".to_string();
        } else {
            self.save_status = "Hack a server during tutorial 2 to unlock Editor.".to_string();
        }
    }
}
