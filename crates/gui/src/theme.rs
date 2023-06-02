use eframe::egui::style::{WidgetVisuals, Widgets};
use eframe::egui::{Color32, Rounding, Stroke, Style, Visuals};
use eframe::epaint::Shadow;

pub fn build_style() -> Style {
    Style {
        visuals: Visuals {
            slider_trailing_fill: true,
            collapsing_header_frame: true,
            menu_rounding: Rounding::same(2.0),
            popup_shadow: Shadow {
                extrusion: 8.0,
                ..Shadow::default()
            },
            window_rounding: Rounding::same(2.0),
            window_shadow: Shadow {
                extrusion: 12.0,
                ..Shadow::default()
            },
            widgets: Widgets {
                hovered: WidgetVisuals {
                    weak_bg_fill: Color32::from_gray(70),
                    bg_fill: Color32::from_gray(70),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                    fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                    rounding: Rounding::same(2.0),
                    expansion: 1.0,
                },
                ..Widgets::default()
            },
            ..Visuals::default()
        },
        ..Style::default()
    }
}
