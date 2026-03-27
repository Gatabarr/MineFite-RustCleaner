fn main() {
    slint_build::compile("ui/main.slint").unwrap();

    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app_icon.ico");
        res.compile().unwrap();
    }
}