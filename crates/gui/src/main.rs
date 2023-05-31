#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::io::BufReader;
use std::path::PathBuf;
use std::{env, fs};

use eframe::egui::Vec2;
use eframe::egui_wgpu::WgpuConfiguration;
use eframe::wgpu;
use eframe::wgpu::PowerPreference;
use poll_promise::Promise;

use hlbc::Bytecode;
use hlbc_gui::App;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "hlbc gui",
        eframe::NativeOptions {
            vsync: true,
            initial_window_size: Some(Vec2::new(1280.0, 720.0)),
            #[cfg(feature = "wgpu")]
            wgpu_options: WgpuConfiguration {
                power_preference: wgpu::util::power_preference_from_env()
                    .unwrap_or(PowerPreference::LowPower),
                ..WgpuConfiguration::default()
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

            // Dock tabs styling
            let style = egui_dock::Style::from_egui(cc.egui_ctx.style().as_ref());

            Box::new(App::new(loader, style))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    //tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "eframe_canvas", // hardcode it
            web_options,
            Box::new(|cc| {
                // Dock tabs styling
                let mut style = egui_dock::Style::from_egui(cc.egui_ctx.style().as_ref());

                Box::new(App {
                    loader: None,
                    ctx: None,
                    tree: Tree::new(vec![]),
                    style,
                    options_window_open: false,
                    about_window_open: false,
                })
            }),
        )
        .await
        .expect("failed to start eframe");
    });
}
