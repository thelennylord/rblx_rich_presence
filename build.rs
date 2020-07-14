#[cfg(target_os = "windows")]
use winres::WindowsResource;

#[cfg(target_os = "windows")]
fn main() {

    // only build for release builds
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut res = WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
}