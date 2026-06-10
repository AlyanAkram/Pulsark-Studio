use eframe::egui;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::file_tree::FileNode;

pub struct Explorer;

impl Explorer {

    pub fn render_tree(
        ui: &mut egui::Ui,
        nodes: &Vec<FileNode>,
        open_files: &mut Vec<PathBuf>,
        active_file: &mut Option<PathBuf>,
        file_contents: &mut HashMap<PathBuf, String>,
        refresh: &mut dyn FnMut(),
    ) {
        ui.push_id("explorer_tree_root", |ui| {

            for node in nodes {
                let name = node.path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                if node.is_dir {

                    ui.push_id(node.path.to_string_lossy(), |ui| {
                        egui::CollapsingHeader::new(name)
                            .show(ui, |ui| {
                                Self::render_tree(
                                    ui,
                                    &node.children,
                                    open_files,
                                    active_file,
                                    file_contents,
                                    refresh,
                                );
                            });
                    });

                } else {
                    let path = node.path.clone();

                    ui.push_id(path.to_string_lossy(), |ui| {

                        let response = ui.button(name);

                        if response.clicked() {
                            if !open_files.contains(&path) {
                                open_files.push(path.clone());

                                let content = std::fs::read_to_string(&path)
                                    .unwrap_or_else(|_| "Failed to read file".to_string());

                                file_contents.insert(path.clone(), content);
                            }

                            *active_file = Some(path.clone());
                        }

                        response.context_menu(|ui| {
                            if ui.button("Delete File").clicked() {
                                let _ = std::fs::remove_file(&path);
                                refresh();
                                ui.close_menu();
                            }

                            if ui.button("Rename (TODO)").clicked() {
                                ui.close_menu();
                            }
                        });
                    });
                }
            }

        });
    }

}