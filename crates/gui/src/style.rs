use std::convert::Into;
use std::sync::OnceLock;

use eframe::egui::text::{LayoutJob, TextWrapping};
use eframe::egui::{
    Color32, FontFamily, FontId, InnerResponse, RichText, ScrollArea, TextStyle, Ui, WidgetText,
};

use crate::model::{AppCtxHandle, Item};

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

pub(crate) fn list_view<Elem: Copy>(
    ui: &mut Ui,
    ctx: AppCtxHandle,
    num: usize,
    item: impl Fn(usize) -> Elem,
    create_selection: impl Fn(Elem) -> Item,
    display: impl Fn(&AppCtxHandle, Elem) -> String,
    context_menu: Option<impl Fn(&mut Ui, &AppCtxHandle, Elem)>,
) {
    ScrollArea::both().auto_shrink([false, false]).show_rows(
        ui,
        ui.text_style_height(&TextStyle::Button),
        num,
        |ui, range| {
            for elem in range.map(item) {
                let checked = ctx.selected() == create_selection(elem);
                let label = ui.selectable_label(
                    checked,
                    singleline(
                        display(&ctx, elem),
                        TextStyle::Button.resolve(ui.style().as_ref()),
                        Color32::WHITE,
                    ),
                );
                if let Some(context_menu) = &context_menu {
                    label.context_menu(|ui| context_menu(ui, &ctx, elem));
                }
                if label.clicked() {
                    ctx.set_selected(create_selection(elem));
                }
            }
        },
    );
}

/// White plain single line text without wrapping and with ellipsis
pub(crate) fn singleline_simple(ui: &Ui, text: impl Into<String>) -> LayoutJob {
    singleline(
        text,
        TextStyle::Body.resolve(ui.style().as_ref()),
        Color32::WHITE,
    )
}

/// Single line text without wrapping and with ellipsis
pub(crate) fn singleline(text: impl Into<String>, font_id: FontId, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::simple_singleline(text.into(), font_id, color);
    job.wrap = TextWrapping {
        break_anywhere: true,
        max_rows: 1,
        ..TextWrapping::default()
    };
    job
}

/// Components stitched together on a single line horizontally.
/// Spacing should be equivalent to a single space character.
pub(crate) fn text_stitch<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        add_contents(ui)
    })
}
