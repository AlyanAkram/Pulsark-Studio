use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::highlight_cache::HighlightCache;
use crate::highlighter::Highlighter;

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
                            let name = file
                                .file_name()
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
        highlighter: &Highlighter,
        cache: &mut HighlightCache,
    ) {
        if let Some(file) = active_file {
            if let Some(content) = file_contents.get_mut(file) {
                // Use extension when present; fall back to full filename
                // so extensionless files (Dockerfile, Makefile, etc.) are detected.
                let extension = file
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_else(|| {
                        file.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                    });

                let line_count = content.lines().count().max(1);

                // Build the line number string once
                let mut line_number_text = (1..=line_count)
                    .map(|i| format!("{:>4}", i))
                    .collect::<Vec<_>>()
                    .join("\n");

                // Gutter layouter — plain monospace, no syntax colors,
                // but identical font/wrap pipeline to the editor layouter.
                let mut gutter_layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    let mut job = highlighter.highlight(text, "");
                    job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(job))
                };

                // Editor layouter with syntax highlighting + cache
                let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    let mut job = cache.get_or_insert(
                        file,
                        text,
                        || highlighter.highlight(text, extension),
                    );
                    job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(job))
                };

                // Remove spacing so the scroll area fills the panel edge-to-edge vertically
                ui.spacing_mut().item_spacing.y = 0.0;

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Zero out internal spacing so line numbers and text sit flush
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

                        ui.horizontal(|ui| {
                            // Line numbers — same layouter pipeline as the editor
                            // so font metrics and row height are guaranteed identical.
                            ui.add(
                                egui::TextEdit::multiline(&mut line_number_text)
                                    .font(egui::TextStyle::Monospace)
                                    .layouter(&mut gutter_layouter)
                                    .desired_width(40.0)
                                    .interactive(false)
                                    .frame(false),
                            );

                            ui.separator();

                            // Editor
                            ui.add(
                                egui::TextEdit::multiline(content)
                                    .font(egui::TextStyle::Monospace)
                                    .layouter(&mut layouter)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                    });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No file selected");
            });
        }
    }
}