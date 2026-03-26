use crate::{AppWindow, SettingsDialog};
use crate::config;
use slint::ComponentHandle;
use std::path::PathBuf;
use std::rc::Rc;

pub fn setup(ui: &AppWindow) {
    let ui_weak = ui.as_weak();
    let selected_file = Rc::new(std::cell::RefCell::new(None::<PathBuf>));

    let cfg = config::load_config();
    ui.set_add_spaces_between_lines(cfg.add_spaces);
    ui.set_remove_non_rp(cfg.remove_non_rp);
    ui.set_use_base_list(cfg.use_base_list);

    let ui_move = ui_weak.clone();
    ui.on_move_window(move |offset_x, offset_y| {
        let ui = ui_move.upgrade().unwrap();
        let pos = ui.window().position();
        ui.window().set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(
            pos.x as f32 + offset_x,
            pos.y as f32 + offset_y,
        )));
    });

    let ui_handle = ui_weak.clone();
    ui.on_open_settings(move || {
        let main_ui = ui_handle.upgrade().unwrap();
        let settings = SettingsDialog::new().expect("Err");
        let cfg = config::load_config();

        settings.set_current_path(cfg.save_path.clone().into());
        settings.set_excluded_tags(cfg.excluded_words.join(", ").into());

        let m_pos = main_ui.window().position();
        let m_size = main_ui.window().size();
        let s_size = settings.window().size();
        settings.window().set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(
            (m_pos.x + (m_size.width as i32 - s_size.width as i32) / 2) as f32,
            (m_pos.y + (m_size.height as i32 - s_size.height as i32) / 2) as f32
        )));

        let s_ptr_move = settings.as_weak();
        settings.on_move_window(move |offset_x, offset_y| {
            let s = s_ptr_move.upgrade().unwrap();
            let pos = s.window().position();
            s.window().set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(
                pos.x as f32 + offset_x,
                pos.y as f32 + offset_y,
            )));
        });

        let s_ptr_for_select = settings.as_weak();
        settings.on_select_path(move || {
            if let Some(s) = s_ptr_for_select.upgrade() {
                if let Some(p) = rfd::FileDialog::new().pick_folder() {
                    s.set_current_path(p.to_string_lossy().to_string().into());
                }
            }
        });

        let s_ptr_for_save = settings.as_weak();
        let ui_handle_for_save = ui_handle.clone();

        settings.on_save_and_close(move |_| {
            let s = s_ptr_for_save.upgrade().expect("Closed");
            let m = ui_handle_for_save.upgrade().expect("Main UI lost");
            
            let path = s.get_current_path().to_string();
            let tags = s.get_excluded_tags().to_string();

            let words: Vec<String> = tags.split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect();

            config::save_config(&config::Config {
                save_path: path,
                add_spaces: m.get_add_spaces_between_lines(),
                remove_non_rp: m.get_remove_non_rp(),
                use_base_list: m.get_use_base_list(),
                excluded_words: words,
            });

            s.hide().unwrap();
        });

        settings.show().unwrap();
    });

    let ui_clean = ui_weak.clone();
    let file_to_clean = selected_file.clone();
    ui.on_request_cleanup(move || {
        if let Some(u) = ui_clean.upgrade() {
            let file_path = file_to_clean.borrow().clone();
            
            if let Some(path) = file_path {
                u.set_processing(true);
                
                let cfg = config::load_config();
                let spaces = u.get_add_spaces_between_lines();
                let nonrp = u.get_remove_non_rp();
                let use_base = u.get_use_base_list();

                let u_done = ui_clean.clone();

                crate::cleaner::process_log_file(
                    path, 
                    cfg.save_path, 
                    spaces, 
                    nonrp, 
                    use_base, 
                    cfg.excluded_words, 
                    move |res| {
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = u_done.upgrade() {
                                ui.set_processing(false);
                                match res {
                                    Ok(r) => {
                                        ui.set_line_count(r.lines_count.to_string().into());
                                        ui.set_log_preview_text(r.preview.into());
                                        ui.set_status_msg("Готово!".into());
                                    }
                                    Err(e) => ui.set_status_msg(e.into()),
                                }
                            }
                        }).unwrap();
                    }
                );
            }
        }
    });

    let ui_c = ui_weak.clone();
    ui.on_close_window(move || { if let Some(u) = ui_c.upgrade() { u.hide().unwrap(); } });
    
    let ui_s = ui_weak.clone();
    let f_s = selected_file.clone();
    ui.on_select_file(move || {
        if let Some(p) = rfd::FileDialog::new().add_filter("Logs", &["log", "txt", "gz"]).pick_file() {
            if let Some(u) = ui_s.upgrade() {
                u.set_file_name(p.file_name().unwrap().to_string_lossy().to_string().into());
                *f_s.borrow_mut() = Some(p);
            }
        }
    });
}