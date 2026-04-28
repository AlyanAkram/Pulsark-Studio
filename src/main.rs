use eframe::egui;
use notify::{Watcher, RecursiveMode, recommended_watcher};
use std::sync::mpsc::channel;
use std::collections::HashMap;


fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Pulsark Studio",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

use std::path::PathBuf;
#[derive(Clone)]
struct FileNode {
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
}


struct MyApp {
    sidebar_width: f32,
    right_panel_width: f32,
    bottom_panel_height: f32,

    open_files: Vec<PathBuf>,
    active_file: Option<PathBuf>,
    file_contents: HashMap<PathBuf, String>,

    current_dir: Option<PathBuf>,
    file_tree: Vec<FileNode>,

    watcher: Option<notify::RecommendedWatcher>,
    receiver: Option<std::sync::mpsc::Receiver<notify::Result<notify::Event>>>,
}


impl Default for MyApp {
    fn default() -> Self {
        Self {
            sidebar_width: 200.0,
            right_panel_width: 250.0,
            bottom_panel_height: 200.0,
            current_dir: None,
            file_tree: vec![],

            open_files: vec![],
            active_file: None,
            file_contents: HashMap::new(),

            watcher: None,
            receiver: None,
        }
    }
}


// 👇 ADD THIS BLOCK RIGHT HERE
impl MyApp {
    fn build_tree(&self, path: &PathBuf) -> Vec<FileNode> {
        let mut nodes = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip unwanted folders
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy();
                    if name == "node_modules" || name == ".git" || name == "target" {
                        continue;
                    }
                }

                let is_dir = path.is_dir();

                let children = if is_dir {
                    self.build_tree(&path)
                } else {
                    vec![]
                };

                nodes.push(FileNode {
                    path,
                    is_dir,
                    children,
                });
            }
        }

        nodes
    }

    fn render_tree(&mut self, ui: &mut egui::Ui, nodes: &Vec<FileNode>) {
        for node in nodes {

            let name = node.path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if node.is_dir {

                let response = egui::CollapsingHeader::new(name.clone())
                    .default_open(false)
                    .show(ui, |ui| {
                        self.render_tree(ui, &node.children);
                    });

                // 👇 Right-click menu for folders
                response.header_response.context_menu(|ui| {
                    if ui.button("New File").clicked() {
                        self.create_file(&node.path);
                        self.refresh_tree();
                        ui.close_menu();
                    }

                    if ui.button("New Folder").clicked() {
                        self.create_folder(&node.path);
                        self.refresh_tree();
                        ui.close_menu();
                    }

                    if ui.button("Delete Folder").clicked() {
                        self.delete_path(&node.path);
                        self.refresh_tree();
                        ui.close_menu();
                    }
                });

            } else {
                let path = node.path.clone();

                let response = ui.button(name.clone());

                // Add to open tabs if not already
                if !self.open_files.contains(&path) {
                    self.open_files.push(path.clone());

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        self.file_contents.insert(path.clone(), content);
                    } else {
                        self.file_contents.insert(path.clone(), "Failed to read file".to_string());
                    }
                }

                // Set active file ONLY when clicked
                if response.clicked() {
                    self.active_file = Some(path.clone());
                }

                // Right-click menu for files
                response.context_menu(|ui| {
                    if ui.button("Delete File").clicked() {
                        self.delete_path(&path);
                        self.refresh_tree();
                        ui.close_menu();
                    }
                });
            }
        }
    }

    fn start_watching(&mut self, path: PathBuf) {

        let (tx, rx) = channel();

        let mut watcher = recommended_watcher(move |res| {
            tx.send(res).unwrap();
        }).unwrap();

        watcher.watch(&path, RecursiveMode::Recursive).unwrap();

        self.watcher = Some(watcher);
        self.receiver = Some(rx);
    }


    fn create_file(&mut self, path: &PathBuf) {
        let new_path = path.join("new_file.txt");
        let _ = std::fs::write(new_path, "");
    }

    fn create_folder(&mut self, path: &PathBuf) {
        let new_path = path.join("new_folder");
        let _ = std::fs::create_dir(new_path);
    }

    fn delete_path(&mut self, path: &PathBuf) {
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(path);
        } else {
            let _ = std::fs::remove_file(path);
        }
    }

    fn refresh_tree(&mut self) {
        if let Some(dir) = &self.current_dir {
            self.file_tree = self.build_tree(dir);
        }
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Check for file system events and refresh tree if needed
        let mut should_refresh = false;

        if let Some(rx) = &self.receiver {
            while let Ok(_event) = rx.try_recv() {
                should_refresh = true;
            }
        }

        if should_refresh {
            self.refresh_tree();
        }


        // 🔹 LEFT SIDEBAR
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(self.sidebar_width)
            .show(ctx, |ui| {

                ui.heading("📁 Explorer");
                ui.separator();

                // Open folder button
                if ui.button("Open Folder").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.current_dir = Some(folder.clone());

                        self.file_tree = self.build_tree(&folder);
                        self.start_watching(folder.clone());
                    }
                }

                ui.separator();

                // Show files if a folder is selected
                if let Some(dir) = &self.current_dir {
                    ui.label(format!("Workspace: {}", dir.display()));

                    let tree = self.file_tree.clone();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.render_tree(ui, &tree);
                    });

                } else {
                    ui.label("No folder opened");
                }
            });


        // 🔹 RIGHT PANEL (AI PANEL)
        egui::SidePanel::right("ai_panel")
            .resizable(true)
            .default_width(self.right_panel_width)
            .show(ctx, |ui| {
                ui.heading("🤖 AI");
                ui.separator();
                ui.label("AI panel coming soon...");
            });

        // 🔹 BOTTOM PANEL (TERMINAL)
        egui::TopBottomPanel::bottom("terminal")
            .resizable(true)
            .default_height(self.bottom_panel_height)
            .show(ctx, |ui| {
                ui.heading("🖥 Terminal");
                ui.separator();
                ui.label("Terminal coming soon...");
            });

        // 🔹 CENTRAL EDITOR AREA
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    if let Some(file) = &self.active_file {
                        if let Some(content) = self.file_contents.get(file) {
                            let _ = std::fs::write(file, content);
                        }
                    }
                }

                if let Some(file) = &self.active_file {
                    ui.label(format!("Editing: {}", file.display()));
                }

                for file in &self.open_files {
                    let name = file.file_name().unwrap().to_string_lossy();

                    let is_active = Some(file) == self.active_file.as_ref();

                    let button = egui::SelectableLabel::new(is_active, name.to_string());

                    if ui.add(button).clicked() {
                        self.active_file = Some(file.clone());
                    }
                }
            });

            ui.heading("Editor");
            ui.separator();

            if let Some(file) = &self.active_file {

                if let Some(content) = self.file_contents.get_mut(file) {

                    egui::ScrollArea::both().show(ui, |ui| {
                        let available_size = ui.available_size();

                        ui.add_sized(
                            available_size,
                            egui::TextEdit::multiline(content)
                                .font(egui::TextStyle::Monospace)
                                .lock_focus(true),
                        );
                    });
                }

            } else {
                ui.label("No file selected");
            }
        });
    }
}
