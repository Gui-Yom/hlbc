use eframe::egui::{FontDefinitions, FontFamily};

pub(crate) fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let custom = egui_ui_refresh::fonts::fonts();

    for (name, data) in custom.font_data {
        fonts.font_data.insert(name, data);
    }

    for (family, custom_fonts) in custom.families {
        match fonts.families.get_mut(&family) {
            Some(existing) => {
                let mut merged = custom_fonts;
                for name in existing.iter() {
                    if !merged.contains(name) {
                        merged.push(name.clone());
                    }
                }
                *existing = merged;
            }
            None => {
                fonts.families.insert(family, custom_fonts);
            }
        }
    }

    if !fonts.families.contains_key(&FontFamily::Name("icons".into())) {
        fonts.families.insert(
            FontFamily::Name("icons".into()),
            vec!["Phosphor".to_owned()],
        );
    }

    fonts
}
