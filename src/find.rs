use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

// ─────────────────────────────────────────────
// State
// ─────────────────────────────────────────────

pub struct FindState {
    pub visible: bool,
    pub query: String,
    pub current_match: usize,
    /// Byte offset of the current match, kept in sync every frame.
    pub current_byte: Option<usize>,
    /// Set to true by navigation actions; consumed (set false) by take_scroll().
    scroll_pending: bool,
}

impl Default for FindState {
    fn default() -> Self {
        Self {
            visible: false,
            query: String::new(),
            current_match: 0,
            current_byte: None,
            scroll_pending: false,
        }
    }
}

impl FindState {
    pub fn open(&mut self) {
        self.visible = true;
        self.current_match = 0;
        self.current_byte = None;
        self.scroll_pending = false;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.current_byte = None;
        self.scroll_pending = false;
    }

    /// The query the editor should use for highlighting.
    /// Returns "" when hidden so highlights disappear immediately.
    pub fn active_query(&self) -> &str {
        if self.visible { &self.query } else { "" }
    }

    /// Call once per frame from app.rs.
    /// Returns the byte offset to scroll to, then clears the one-shot flag.
    pub fn take_scroll(&mut self) -> Option<usize> {
        if self.scroll_pending {
            self.scroll_pending = false;
            self.current_byte
        } else {
            None
        }
    }
}

// ─────────────────────────────────────────────
// Panel
// ─────────────────────────────────────────────

pub struct FindPanel;

impl FindPanel {
    pub fn show(
        ctx: &egui::Context,
        state: &mut FindState,
        editor_rect: egui::Rect,
        file_contents: &HashMap<PathBuf, String>,
        active_file: &Option<PathBuf>,
    ) {
        if !state.visible {
            return;
        }

        let match_offsets = Self::collect_matches(state.query.as_str(), file_contents, active_file);
        let match_count = match_offsets.len();

        // Keep current_match in bounds and update current_byte
        if match_count == 0 {
            state.current_match = 0;
            state.current_byte = None;
        } else {
            state.current_match = state.current_match.min(match_count - 1);
            state.current_byte = Some(match_offsets[state.current_match]);
        }

        let panel_width = 340.0;
        let margin = 8.0;
        let pos = egui::pos2(
            editor_rect.right() - panel_width - margin,
            editor_rect.top() + margin,
        );

        let mut close_requested = false;
        let old_query = state.query.clone();

        egui::Area::new(egui::Id::new("find_overlay"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::window(ui.style()).show(ui, |ui| {
                    ui.set_width(panel_width);

                    ui.horizontal(|ui| {
                        ui.label("Find:");

                        let response = ui.add(
                            egui::TextEdit::singleline(&mut state.query)
                                .hint_text("search…")
                                .desired_width(180.0),
                        );

                        if !response.has_focus() {
                            response.request_focus();
                        }

                        if state.query.is_empty() {
                            ui.label("–");
                        } else if match_count == 0 {
                            ui.colored_label(egui::Color32::RED, "0/0");
                        } else {
                            ui.label(format!("{}/{}", state.current_match + 1, match_count));
                        }

                        let prev_clicked = ui
                            .add_enabled(match_count > 0, egui::Button::new("< Prev"))
                            .clicked();

                        let next_clicked = ui
                            .add_enabled(match_count > 0, egui::Button::new("Next >"))
                            .clicked();

                        if ui.button("X").clicked() {
                            close_requested = true;
                        }

                        let f3 = ctx.input(|i| i.key_pressed(egui::Key::F3) && !i.modifiers.shift);
                        let shift_f3 = ctx.input(|i| i.key_pressed(egui::Key::F3) && i.modifiers.shift);
                        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
                        let shift_enter = ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.shift);

                        if match_count > 0 {
                            if prev_clicked || shift_f3 || shift_enter {
                                state.current_match =
                                    (state.current_match + match_count - 1) % match_count;
                                state.current_byte = Some(match_offsets[state.current_match]);
                                state.scroll_pending = true;
                            }
                            if next_clicked || f3 || enter {
                                state.current_match = (state.current_match + 1) % match_count;
                                state.current_byte = Some(match_offsets[state.current_match]);
                                state.scroll_pending = true;
                            }
                        }
                    });
                });
            });

        // Query changed → reset to first match and scroll to it
        if state.query != old_query {
            state.current_match = 0;
            state.scroll_pending = match_count > 0;
        }

        if close_requested {
            state.close();
        }
    }

    pub fn collect_matches(
        query: &str,
        file_contents: &HashMap<PathBuf, String>,
        active_file: &Option<PathBuf>,
    ) -> Vec<usize> {
        if query.is_empty() {
            return vec![];
        }
        let Some(file) = active_file else { return vec![] };
        let Some(content) = file_contents.get(file) else { return vec![] };

        let lower_content = content.to_lowercase();
        let lower_query = query.to_lowercase();
        let qlen = lower_query.len();

        let mut offsets = Vec::new();
        let mut pos = 0usize;
        while let Some(rel) = lower_content[pos..].find(lower_query.as_str()) {
            offsets.push(pos + rel);
            pos += rel + qlen;
        }
        offsets
    }
}