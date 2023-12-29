#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};

use eframe::egui::{IconData, Vec2, ViewportBuilder};
use image::ImageFormat;
use poll_promise::Promise;

use hlbc::Bytecode;
use hlbc_gui::{App, HLBC_ICON};

#[cfg(not(target_arch = "wasm32"))]
mod image_loader;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let icon = image::load_from_memory_with_format(HLBC_ICON, ImageFormat::Ico).unwrap();
    let icon_data = Arc::new(IconData {
        width: icon.width(),
        height: icon.height(),
        rgba: icon.into_bytes(),
    });
    eframe::run_native(
        "hlbc",
        eframe::NativeOptions {
            vsync: true,
            viewport: ViewportBuilder::default()
                .with_inner_size(Vec2::new(1280.0, 720.0))
                .with_icon(icon_data),
            #[cfg(feature = "wgpu")]
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                power_preference: eframe::wgpu::util::power_preference_from_env()
                    .unwrap_or(eframe::wgpu::PowerPreference::LowPower),
                ..eframe::egui_wgpu::WgpuConfiguration::default()
            },
            ..Default::default()
        },
        Box::new(|cc| {
            let args = env::args().skip(1).collect::<String>();
            let loader = if args.is_empty() {
                None
            } else {
                let path = PathBuf::from(args.clone());
                Some(Promise::spawn_thread("bg_loader", move || {
                    Ok(Some((
                        args,
                        Bytecode::deserialize(&mut BufReader::new(fs::File::open(&path)?))?,
                    )))
                }))
            };
            cc.egui_ctx
                .add_image_loader(Arc::new(image_loader::ImageCrateLoader::default()));

            cc.egui_ctx.set_fonts(egui_ui_refresh::fonts());
            cc.egui_ctx.set_style(egui_ui_refresh::style());

            // Dock tabs styling
            let style = egui_dock::Style::from_egui(cc.egui_ctx.style().as_ref());

            Box::new(App::new(loader, style))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    // eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "eframe_canvas", // hardcode it
                web_options,
                Box::new(|cc| {
                    cc.egui_ctx.set_fonts(egui_ui_refresh::fonts());
                    cc.egui_ctx.set_style(egui_ui_refresh::style());

                    // Dock tabs styling
                    let mut style = egui_dock::Style::from_egui(cc.egui_ctx.style().as_ref());

                    Box::new(App::new(None, style))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
