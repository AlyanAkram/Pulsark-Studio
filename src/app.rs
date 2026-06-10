use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::file_tree;
use crate::file_tree::FileNode;

use crate::explorer::Explorer;
use crate::editor::Editor;

pub struct MyApp {
    pub sidebar_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,

    pub show_explorer: bool,
    pub show_ai: bool,
    pub show_terminal: bool,

    pub open_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub file_contents: HashMap<PathBuf, String>,

    pub current_dir: Option<PathBuf>,
    pub file_tree: Vec<FileNode>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            sidebar_width: 200.0,
            right_panel_width: 250.0,
            bottom_panel_height: 200.0,

            show_explorer: true,
            show_ai: true,
            show_terminal: true,

            open_files: vec![],
            active_file: None,
            file_contents: HashMap::new(),

            current_dir: None,
            file_tree: vec![],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {

            ui.horizontal(|ui| {

                ui.menu_button("View", |ui| {

                    if ui.checkbox(&mut self.show_explorer, "Explorer").clicked() {
                        ui.close_menu();
                    }

                    if ui.checkbox(&mut self.show_ai, "AI Panel").clicked() {
                        ui.close_menu();
                    }

                    if ui.checkbox(&mut self.show_terminal, "Terminal").clicked() {
                        ui.close_menu();
                    }

                });

            });

        });

        // =========================
        // LEFT SIDEBAR (EXPLORER)
        // =========================
        if self.show_explorer {

            egui::SidePanel::left("sidebar")
                .resizable(true)
                .default_width(self.sidebar_width)
                .show(ctx, |ui| {

                    ui.heading("📁 Explorer");
                    ui.separator();

                    // Open folder
                    if ui.button("Open Folder").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.current_dir = Some(folder.clone());
                            self.file_tree = file_tree::build_tree(&folder);
                        }
                    }

                    ui.separator();

                    if let Some(dir) = &self.current_dir {
                        ui.label(format!("Workspace: {}", dir.display()));

                        let tree = self.file_tree.clone();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            Explorer::render_tree(
                                ui,
                                &tree,
                                &mut self.open_files,
                                &mut self.active_file,
                                &mut self.file_contents,
                                &mut || {
                                    if let Some(dir) = &self.current_dir {
                                        self.file_tree = file_tree::build_tree(dir);
                                    }
                                },
                            );
                        });
                    } else {
                        ui.label("No folder opened");
                    }
                });
        }

        // =========================
        // RIGHT PANEL (AI)
        // =========================
        if self.show_ai {

            egui::SidePanel::right("ai_panel")
                .resizable(true)
                .default_width(self.right_panel_width)
                .show(ctx, |ui| {
                    ui.heading("🤖 AI");
                    ui.separator();
                    ui.label("AI panel coming soon...");
            });
        }

        // =========================
        // BOTTOM PANEL (TERMINAL)
        // =========================
        if self.show_terminal {

            egui::TopBottomPanel::bottom("terminal")
                .resizable(true)
                .default_height(self.bottom_panel_height)
                .show(ctx, |ui| {
                    ui.heading("🖥 Terminal");
                    ui.separator();
                    ui.label("Terminal coming soon...");
            });
        }

        // =========================
        // CENTRAL EDITOR
        // =========================
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    if let Some(file) = &self.active_file {
                        if let Some(content) = self.file_contents.get(file) {
                            let _ = std::fs::write(file, content);
                        }
                    }
                }
            });

            ui.separator();

            ui.heading("Editor");
            ui.separator();

            Editor::draw_tabs(
                ui,
                &mut self.open_files,
                &mut self.active_file,
                &mut self.file_contents,
            );

            ui.separator();

            Editor::draw_editor(
                ui,
                &self.active_file,
                &mut self.file_contents,
            );
        });
    }
}