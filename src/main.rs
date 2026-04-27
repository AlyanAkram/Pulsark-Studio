use eframe::egui;

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

struct MyApp {
    sidebar_width: f32,
    right_panel_width: f32,
    bottom_panel_height: f32,
    selected_file: Option<PathBuf>,

    current_dir: Option<PathBuf>,
    files: Vec<PathBuf>,
}


impl Default for MyApp {
    fn default() -> Self {
        Self {
            sidebar_width: 200.0,
            right_panel_width: 250.0,
            bottom_panel_height: 200.0,
            current_dir: None,
            files: vec![],
            selected_file: None,

        }
    }
}


// 👇 ADD THIS BLOCK RIGHT HERE
impl MyApp {
    fn read_dir_recursive(&mut self, path: &PathBuf) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();

                // 👇 Skip useless/heavy folders
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy();

                    if name == "node_modules" || name == ".git" || name == "target" {
                        continue;
                    }
                }

                self.files.push(path.clone());

                if path.is_dir() {
                    self.read_dir_recursive(&path);
                }
            }
        }
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

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

                        // Clear previous files
                        self.files.clear();

                        // Scan new folder
                        self.read_dir_recursive(&folder);
                    }
                }

                ui.separator();

                // Show files if a folder is selected
                if let Some(dir) = &self.current_dir {
                    ui.label(format!("Workspace: {}", dir.display()));

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for file in &self.files {
                            if file.is_file() {
                                if let Some(name) = file.file_name() {
                                    if ui.button(name.to_string_lossy()).clicked() {
                                        self.selected_file = Some(file.clone());
                                    }
                                }
                            }
                        }
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
            ui.heading("Editor");
            ui.separator();

            if let Some(file) = &self.selected_file {
                ui.label(format!("Opened: {}", file.display()));
            } else {
                ui.label("No file selected");
            }
        });
    }
}
