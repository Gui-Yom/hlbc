use eframe::egui;
use eframe::egui::load::Bytes;
use eframe::egui::{Context, ImageSource};

use crate::views::text_stitch;
use crate::HLBC_ICON;

pub(crate) fn about_window(ctx: &Context, open: &mut bool) {
    egui::Window::new("About")
        .open(open)
        .resizable(false)
        .collapsible(false)
        .fixed_size((300., 200.))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.image(ImageSource::Bytes {
                    uri: "bytes://hlbc.ico".into(),
                    bytes: Bytes::Static(HLBC_ICON),
                });
                ui.heading("Hashlink bytecode tools");
                ui.hyperlink("https://github.com/Gui-Yom/hlbc");
                text_stitch(ui, |ui| {
                    ui.label("Made by");
                    ui.hyperlink_to("Gui-Yom", "https://github.com/Gui-Yom");
                    ui.label("and");
                    ui.hyperlink_to(
                        "contributors",
                        "https://github.com/Gui-Yom/hlbc/graphs/contributors",
                    );
                });
            });
        });
}
