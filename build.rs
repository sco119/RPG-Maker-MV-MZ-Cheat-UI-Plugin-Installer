fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        let copyright_text = format!("Copyright (C) {}", env!("CARGO_PKG_AUTHORS"));
        res.set("LegalCopyright", &copyright_text);
        res.set("FileDescription", "Cheat UI Plugin Installer for RPG Maker MV/MZ Games");
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}