use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let icon_path = "../../assets/hlbc.ico";
        println!("cargo:rerun-if-changed={icon_path}");

        let mut res = winresource::WindowsResource::new();
        res.set_icon(icon_path);
        res.compile().unwrap();
    }
}
