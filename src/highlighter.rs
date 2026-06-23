use eframe::egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
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

    /// Map any file extension (lowercased) to the syntect extension string.
    fn ext_to_syntax(ext: &str) -> &'static str {
        match ext.to_lowercase().as_str() {
            // Rust
            "rs"                         => "rs",
            // Python
            "py" | "pyw" | "pyi"        => "py",
            // JavaScript / TypeScript
            "js" | "mjs" | "cjs"        => "js",
            "jsx"                        => "js",   // syntect has no jsx; js is close
            "ts" | "mts" | "cts"        => "ts",
            "tsx"                        => "ts",
            // Web
            "html" | "htm" | "xhtml"    => "html",
            "css"                        => "css",
            "scss"                       => "scss",
            "sass"                       => "sass",
            "less"                       => "less",
            // Data / config
            "json" | "jsonc"             => "json",
            "yaml" | "yml"              => "yaml",
            "toml"                       => "toml",
            "xml" | "svg" | "plist"     => "xml",
            "ini" | "cfg" | "conf"      => "ini",
            "env"                        => "ini",
            // Shell
            "sh" | "bash" | "zsh" | "ksh" => "sh",
            "fish"                       => "fish",
            "ps1" | "psm1" | "psd1"     => "ps1",
            "bat" | "cmd"               => "bat",
            // Systems
            "c"                          => "c",
            "h"                          => "c",
            "cpp" | "cxx" | "cc" | "c++" => "cpp",
            "hpp" | "hxx" | "hh"        => "cpp",
            "cs"                         => "cs",
            "java"                       => "java",
            "kt" | "kts"                => "kt",
            "scala" | "sc"              => "scala",
            "swift"                      => "swift",
            "go"                         => "go",
            // Scripting
            "rb" | "rake" | "gemspec"   => "rb",
            "php" | "php3" | "php4" | "php5" => "php",
            "lua"                        => "lua",
            "pl" | "pm"                 => "pl",
            "r"                          => "r",
            "m"                          => "m",   // Objective-C / MATLAB
            // DB
            "sql"                        => "sql",
            // Markup / docs
            "md" | "markdown"           => "md",
            "rst"                        => "rst",
            "tex" | "latex"             => "tex",
            // DevOps / infra
            "tf" | "tfvars"             => "tf",
            "proto"                      => "proto",
            "graphql" | "gql"           => "graphql",
            "vue"                        => "html",  // close enough; no Vue syntax bundled
            "dockerfile"                 => "dockerfile",
            "makefile"                   => "makefile",
            // Catch-all: return empty and let syntect try by name
            _                            => "",
        }
    }

    pub fn highlight(&self, text: &str, extension: &str) -> LayoutJob {
        // Allow callers to pass the full filename (for extensionless files)
        // or just the extension.
        let resolved = if extension.is_empty() {
            ""
        } else {
            Self::ext_to_syntax(extension)
        };

        let syntax = if resolved.is_empty() {
            self.ps.find_syntax_plain_text()
        } else {
            self.ps
                .find_syntax_by_extension(resolved)
                .or_else(|| self.ps.find_syntax_by_name(resolved))
                .unwrap_or_else(|| self.ps.find_syntax_plain_text())
        };

        let mut h = HighlightLines::new(syntax, &self.theme);
        let mut job = LayoutJob::default();

        // LinesWithEndings preserves the \n on each line, which syntect
        // needs to correctly close line-scoped tokens (e.g. Python comments).
        // Using text.lines() strips the newline, leaving comment scopes open
        // and causing everything after a # to be coloured as a comment.
        for line in LinesWithEndings::from(text) {
            let ranges = h.highlight_line(line, &self.ps).unwrap_or_default();

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
        }

        job
    }
}