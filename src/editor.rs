use eframe::egui;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct Editor;

impl Editor {

    pub fn draw_tabs(
        ui: &mut egui::Ui,
        open_files: &mut Vec<PathBuf>,
        active_file: &mut Option<PathBuf>,
        file_contents: &mut HashMap<PathBuf, String>,
    ) {
        ui.push_id("tab_bar", |ui| {
            egui::ScrollArea::horizontal()
                .max_height(32.0)
                .show(ui, |ui| {

                    ui.horizontal(|ui| {

                        let mut to_close = None;

                        for file in open_files.clone() {

                            let name = file.file_name()
                                .unwrap_or_default()
                                .to_string_lossy();

                            let is_active = Some(&file) == active_file.as_ref();

                            ui.horizontal(|ui| {

                                if ui.selectable_label(is_active, name).clicked() {
                                    *active_file = Some(file.clone());
                                }

                                if ui.small_button("x").clicked() {
                                    to_close = Some(file.clone());
                                }
                            });
                        }

                        if let Some(file) = to_close {
                            open_files.retain(|f| f != &file);
                            file_contents.remove(&file);

                            if active_file.as_ref() == Some(&file) {
                                *active_file = open_files.last().cloned();
                            }
                        }
                    });
                });
        });
    }

    pub fn draw_editor(
        ui: &mut egui::Ui,
        active_file: &Option<PathBuf>,
        file_contents: &mut HashMap<PathBuf, String>,
    ) {
        if let Some(file) = active_file {
            if let Some(content) = file_contents.get_mut(file) {

                egui::ScrollArea::both().show(ui, |ui| {
                    let size = ui.available_size();

                    ui.add_sized(
                        size,
                        egui::TextEdit::multiline(content)
                            .font(egui::TextStyle::Monospace),
                    );
                });

            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No file selected");
            });
        }
    }
}