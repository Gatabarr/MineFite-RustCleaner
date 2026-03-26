#![windows_subsystem = "windows"]

mod config;
mod ui_bridge;
mod cleaner;

use slint::ComponentHandle;
use std::fs;
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
slint::include_modules!();

fn check_database_update(ui_handle: slint::Weak<AppWindow>) {
    let raw_url = "https://raw.githubusercontent.com/Gatabarr/logcleaner-baselist/main/base_list.txt";
    let local_path = crate::config::get_app_dir().join("base_list.txt");

    thread::spawn(move || {
        let cache_buster = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let url = format!("{}?t={}", raw_url, cache_buster);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        ui_handle.upgrade_in_event_loop(|ui| {
            ui.set_is_updating(true);
            ui.set_update_status("Синхронизация базы...".into());
        }).ok();

        if let Ok(mut response) = client.get(url).send() {
            if response.status().is_success() {
                let total_size = response.content_length().unwrap_or(1);
                let mut downloaded: u64 = 0;
                let mut buffer = [0; 1024];
                let mut new_content = Vec::new();

                while let Ok(n) = response.read(&mut buffer) {
                    if n == 0 { break; }
                    new_content.extend_from_slice(&buffer[..n]);
                    downloaded += n as u64;
                    let progress = downloaded as f32 / total_size as f32;
                    ui_handle.upgrade_in_event_loop(move |ui| {
                        ui.set_update_progress(progress);
                    }).ok();
                }

                if let Ok(new_text) = String::from_utf8(new_content) {
                    let current_text = fs::read_to_string(&local_path).unwrap_or_default();

                    if new_text.trim() != current_text.trim() {
                        ui_handle.upgrade_in_event_loop(|ui| {
                            ui.set_update_status("Обновление файлов...".into());
                        }).ok();

                        if fs::write(&local_path, &new_text).is_ok() {
                            ui_handle.upgrade_in_event_loop(|ui| {
                                ui.set_update_status("Готово! Перезапуск...".into());
                            }).ok();
                            
                            thread::sleep(Duration::from_secs(1));
                            
                            if let Ok(exe) = std::env::current_exe() {
                                let _ = std::process::Command::new(exe).spawn();
                                std::process::exit(0);
                            }
                        }
                    }
                }
            }
        }
        ui_handle.upgrade_in_event_loop(|ui| ui.set_is_updating(false)).ok();
    });
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    check_database_update(ui.as_weak());
    ui_bridge::setup(&ui);
    ui.run()
}