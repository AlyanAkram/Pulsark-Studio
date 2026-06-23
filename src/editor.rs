use eframe::egui::{self, text::LayoutJob, Color32, TextFormat, FontId};
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

    /// Takes the syntax-highlighted LayoutJob and overlays match highlights on top.
    /// `current_match` is 0-indexed; that match gets a brighter highlight than the rest.
    fn apply_find_highlights(
        base_job: LayoutJob,
        text: &str,
        query: &str,
        current_match: usize,
    ) -> LayoutJob {
        if query.is_empty() {
            return base_job;
        }

        let lower_text  = text.to_lowercase();
        let lower_query = query.to_lowercase();

        // Collect all match byte ranges
        let mut matches: Vec<usize> = Vec::new();
        let mut search_from = 0;
        while let Some(rel) = lower_text[search_from..].find(lower_query.as_str()) {
            let start = search_from + rel;
            matches.push(start);
            search_from = start + lower_query.len();
        }

        if matches.is_empty() {
            return base_job;
        }

        let mut new_job = LayoutJob::default();
        new_job.wrap = base_job.wrap.clone();

        let qlen = lower_query.len();

        let match_ranges: Vec<(usize, usize, bool)> = matches
            .iter()
            .enumerate()
            .map(|(i, &s)| (s, s + qlen, i == current_match))
            .collect();

        for section in &base_job.sections {
            let sec_start = section.byte_range.start;
            let sec_end   = section.byte_range.end;
            let base_fmt  = &section.format;

            let mut cursor = sec_start;

            for &(m_start, m_end, is_current) in &match_ranges {
                if m_end <= sec_start || m_start >= sec_end {
                    continue;
                }

                let eff_start = m_start.max(sec_start);
                let eff_end   = m_end.min(sec_end);

                if cursor < eff_start {
                    new_job.append(&text[cursor..eff_start], 0.0, base_fmt.clone());
                }

                let highlight_bg = if is_current {
                    Color32::from_rgb(255, 140, 0)   // orange – current match
                } else {
                    Color32::from_rgb(80, 80, 0)     // dim yellow – other matches
                };

                new_job.append(
                    &text[eff_start..eff_end],
                    0.0,
                    TextFormat {
                        font_id: FontId::monospace(14.0),
                        color: Color32::WHITE,
                        background: highlight_bg,
                        ..Default::default()
                    },
                );

                cursor = eff_end;
            }

            if cursor < sec_end {
                new_job.append(&text[cursor..sec_end], 0.0, base_fmt.clone());
            }
        }

        new_job
    }

    /// Estimate the vertical pixel offset of a byte position inside `text`.
    ///
    /// Counts actual newlines up to `byte_pos`, then adds extra rows for any
    /// lines that are wider than `wrap_width` characters (soft-wrap).
    /// Using character count as a proxy for pixel width is accurate enough for
    /// monospace fonts where every glyph is the same width.
    fn byte_to_y_offset(text: &str, byte_pos: usize, line_height: f32) -> f32 {
        let safe_pos = byte_pos.min(text.len());
        let lines_above = text[..safe_pos].chars().filter(|&c| c == '\n').count();
        lines_above as f32 * line_height
    }

    pub fn draw_editor(
        ui: &mut egui::Ui,
        active_file: &Option<PathBuf>,
        file_contents: &mut HashMap<PathBuf, String>,
        highlighter: &Highlighter,
        cache: &mut HighlightCache,
        find_query: &str,
        find_current: usize,
        scroll_to_byte: Option<usize>, // byte offset to scroll to THIS FRAME only, or None
    ) {
        if let Some(file) = active_file {
            if let Some(content) = file_contents.get_mut(file) {
                let extension = file
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_else(|| {
                        file.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                    });

                let line_count = content.lines().count().max(1);

                let mut line_number_text = (1..=line_count)
                    .map(|i| format!("{:>4}", i))
                    .collect::<Vec<_>>()
                    .join("\n");

                let mut gutter_layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    let mut job = highlighter.highlight(text, "");
                    job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(job))
                };

                let fq = find_query.to_string();
                let fc = find_current;
                let file_clone = file.clone();

                let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    let mut base = cache.get_or_insert(
                        &file_clone,
                        text,
                        || highlighter.highlight(text, extension),
                    );
                    base.wrap.max_width = wrap_width;

                    let job = if fq.is_empty() {
                        base
                    } else {
                        Self::apply_find_highlights(base, text, &fq, fc)
                    };

                    ui.fonts(|f| f.layout_job(job))
                };

                // Approximate line height and editor column width for monospace 14 px.
                // A monospace char at size 14 is roughly 8.4 px wide; the editor
                // column is the panel width minus the ~50 px gutter.
                const LINE_HEIGHT: f32 = 18.0;

                ui.spacing_mut().item_spacing.y = 0.0;

                let mut scroll_area = egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .id_source("editor_scroll");

                if let Some(byte) = scroll_to_byte {
                    let target_y = Self::byte_to_y_offset(content, byte, LINE_HEIGHT);
                    scroll_area = scroll_area.vertical_scroll_offset((target_y - LINE_HEIGHT * 3.0).max(0.0));
                }

                scroll_area.show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut line_number_text)
                                .font(egui::TextStyle::Monospace)
                                .layouter(&mut gutter_layouter)
                                .desired_width(40.0)
                                .interactive(false)
                                .frame(false),
                        );

                        ui.separator();

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