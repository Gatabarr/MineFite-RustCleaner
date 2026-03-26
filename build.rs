fn main() {
    // 1. Компиляция Slint
    slint_build::compile("ui/main.slint").unwrap();

    // 2. Добавление иконки для Windows
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app_icon.ico"); // Убедись, что файл есть в корне!
        res.compile().unwrap();
    }
}