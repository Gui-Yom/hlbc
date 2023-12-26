use std::convert::Into;
use std::sync::OnceLock;

use eframe::egui::{FontFamily, FontId, RichText, WidgetText};

pub(crate) struct Styles {
    pub(crate) heading_title: FontId,
    pub(crate) heading_subtitle: FontId,
    pub(crate) homepage_button: FontId,
}

pub(crate) fn get() -> &'static Styles {
    static STYLES: OnceLock<Styles> = OnceLock::new();
    STYLES.get_or_init(|| Styles {
        heading_title: FontId::new(50.0, FontFamily::Name(egui_ui_refresh::MEDIUM.into())),
        heading_subtitle: FontId::new(20.0, FontFamily::Name(egui_ui_refresh::ITALIC.into())),
        homepage_button: FontId::new(50.0, FontFamily::Proportional),
    })
}

pub(crate) fn text(text: impl Into<String>, font: FontId) -> impl Into<WidgetText> {
    RichText::new(text).font(font)
}
