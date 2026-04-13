#[cfg(target_os = "windows")]
fn main() {
    let icon_path = "../../assets/hlbc.ico";
    println!("cargo:rerun-if-changed={icon_path}");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon_path);
    res.compile().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {}
