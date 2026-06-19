use eframe::egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};

pub struct Highlighter {
    ps: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new() -> Self {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        Self {
            ps,
            theme: ts.themes["base16-ocean.dark"].clone(),
        }
    }

    pub fn highlight(&self, text: &str, extension: &str) -> LayoutJob {
        let syntax = self
            .ps
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, &self.theme);

        let mut job = LayoutJob::default();

        for (index, line) in text.lines().enumerate() {

            let ranges = h.highlight_line(line, &self.ps)
                .unwrap_or_default();

            for (style, piece) in ranges {

                let color = Color32::from_rgb(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                );

                job.append(
                    piece,
                    0.0,
                    TextFormat {
                        font_id: FontId::monospace(14.0),
                        color,
                        ..Default::default()
                    },
                );
            }

            if index + 1 < text.lines().count() {
                job.append(
                    "\n",
                    0.0,
                    TextFormat {
                        font_id: FontId::monospace(14.0),
                        color: Color32::WHITE,
                        ..Default::default()
                    },
                );
            }
        }

        job
    }
}