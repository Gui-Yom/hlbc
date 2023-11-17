use eframe::egui::text::{LayoutJob, LayoutSection};
use eframe::egui::util::cache::{ComputerMut, FrameCache};
use eframe::egui::{Color32, FontId, Response, Stroke, TextEdit, TextFormat, Ui};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder};
use syntect::util::LinesWithEndings;

pub(crate) fn haxe_source_view(ui: &mut Ui, source: &str) -> Response {
    let mut temp = source;
    ui.add(
        TextEdit::multiline(&mut temp)
            .code_editor()
            .lock_focus(false)
            .layouter(&mut |ui, code, _wrap| {
                let job = {
                    ui.memory_mut(|mem| {
                        let cache = mem.caches.cache::<FrameCache<LayoutJob, Highlighter>>();
                        cache.get(("base16-mocha.dark", code, "hx"))
                    })
                };
                ui.fonts(|fonts| fonts.layout_job(job))
            }),
    )
}

struct Highlighter {
    ps: SyntaxSet,
    ts: ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        let syntax = SyntaxDefinition::load_from_str(
            include_str!("../../assets/Haxe.sublime-syntax"),
            true,
            Some("Haxe"),
        )
        .expect("Can't load syntax definition");

        let mut builder = SyntaxSetBuilder::new();
        builder.add(syntax);

        Self {
            ps: builder.build(),
            ts: ThemeSet::load_defaults(),
        }
    }
}

impl ComputerMut<(&str, &str, &str), LayoutJob> for Highlighter {
    fn compute(&mut self, (theme, code, lang): (&str, &str, &str)) -> LayoutJob {
        let syntax = self
            .ps
            .find_syntax_by_name(lang)
            .or_else(|| self.ps.find_syntax_by_extension(lang))
            .unwrap();

        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        let mut job = LayoutJob {
            text: code.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(code) {
            for (style, range) in h.highlight_line(line, &self.ps).unwrap() {
                let fg = style.foreground;
                let text_color = Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    Stroke::new(1.0, text_color)
                } else {
                    Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(code, range),
                    format: TextFormat {
                        font_id: FontId::monospace(14.0),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        job
    }
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}
