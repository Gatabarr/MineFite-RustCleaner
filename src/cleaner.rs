use std::fs;
use std::path::PathBuf;
use std::thread;
use std::io::Read; 
use flate2::read::GzDecoder;

pub struct CleanResult {
    pub lines_count: usize,
    pub preview: String,
    pub out_path: String,
}

fn get_base_excluded_words() -> Vec<String> {
    let path = crate::config::get_app_dir().join("base_list.txt");
    if !path.exists() {
        let default_content = "
        ► Нажми ◄\n
        Добро пожаловать на материк Миртан\n
        Получить помощь по командам /ehelp. Подать жалобу - /report, задать вопрос - /ask.\n
        Вам доступен Фолиант Знаний! Начните исследовать мир, открыв его по команде /folio.\n
        доступ к важным системам, а так же раздел информации для новичков по пунктам.\n
        Не знаете куда пойти и какие локации есть? Откройте онлайн карту - map.minefite.net.\n
        Новичок в РП или майнкрафте? Откройте наши гайды для новичков в игре! /newinfo\n
        Если живете и играете в городе фракции, купите ключи для закрытия своих сундуков и дверей!\n
        Ключи доступны в крафте по команде /crafting, в разделе прочее!\n
        Доступна новая версия Plasmo Voice\n
        §eJourneyMap:§f Нажмите\n
        Вы не можете использовать\n
        [Земли]\n
        [АФК+]\n
        ПЭЙ ДЭЙ\n
        Откройте наше игровое меню по команде /mn! Там доступны все основные меню и быстрый\n
        [Отладка]\n
        [Оповещение] »\n
        Ты достиг нового уровня и получил очко\n
        Используйте /profile чтобы увидеть новую статистику!\n
        Вы успешно повысили уровень\n
        Поздравляем, вы достигли уровня\n
        [Заведение]\n
        [FeedbackForwardingSender]\n
        временно забанил\n
        Загрузка модели\n
        Ваш рост был изменен\n
        был кикнут администратором";
        let _ = fs::write(&path, default_content);
    }
    
    if let Ok(content) = fs::read_to_string(path) {
        content.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![]
    }
}

pub fn process_log_file(
    input_path: PathBuf,
    save_dir_str: String,
    add_spaces: bool,
    remove_non_rp: bool,
    use_base_list: bool,
    excluded_words: Vec<String>,
    on_done: impl FnOnce(Result<CleanResult, String>) + Send + 'static,
) {
    thread::spawn(move || {
        let save_dir = PathBuf::from(&save_dir_str);
        if !save_dir.exists() { let _ = fs::create_dir_all(&save_dir); }

        let mut final_excluded = excluded_words;
        if use_base_list {
            final_excluded.extend(get_base_excluded_words());
        }

        let raw_file_name = input_path.file_name().unwrap().to_string_lossy();
        let out_name = format!("cleaned_{}", raw_file_name.replace(".gz", ""));
        let out_path = save_dir.join(out_name);

        let content_result = if input_path.extension().and_then(|s| s.to_str()) == Some("gz") {
            fs::read(&input_path).map_err(|_| "Ошибка архива".to_string()).and_then(|bytes| {
                let mut decoder = GzDecoder::new(&bytes[..]);
                let mut s = String::new();
                decoder.read_to_string(&mut s).map(|_| s).map_err(|_| "Не текст внутри GZ".to_string())
            })
        } else {
            fs::read_to_string(&input_path).map_err(|_| "Ошибка чтения файла".to_string())
        };

        match content_result {
            Ok(content) => {
                let mut processed_lines = Vec::new();
                let mut actual_log_count = 0;

                for line in content.lines() {
                    let chat_pos = line.find("[CHAT]");
                    let pm_pos = line.find("[PM]");

                    if chat_pos.is_some() || pm_pos.is_some() {
                        let split_pos = match (chat_pos, pm_pos) {
                            (Some(c), Some(p)) => if c > p { c + 6 } else { p + 4 },
                            (Some(c), None) => c + 6,
                            (None, Some(p)) => p + 4,
                            _ => 0,
                        };

                        if split_pos >= line.len() { continue; }
                        
                        let mut raw_msg = line[split_pos..].trim();
                        if raw_msg.starts_with(':') { raw_msg = raw_msg[1..].trim(); }

                        if !raw_msg.is_empty() {
                            let final_msg = raw_msg.replace("[CHAT]", "").replace("[PM]", "").trim().to_string();
                            if final_msg.is_empty() { continue; }

                            let mut should_skip = false;
                            for word in &final_excluded {
                                let w = word.trim();
                                if w.len() > 2 && final_msg.contains(w) {
                                    should_skip = true;
                                    break;
                                }
                            }
                            if should_skip { continue; }

                            if remove_non_rp && final_msg.starts_with("((") && final_msg.ends_with("))") {
                                continue;
                            }

                            processed_lines.push(final_msg);
                            actual_log_count += 1;


                            if add_spaces {
                                processed_lines.push("".to_string());
                            }
                        }
                    }
                }

                let full_text = processed_lines.join("\n");
                
                let preview = processed_lines.iter()
                    .take(75) 
                    .cloned()
                    .collect::<Vec<String>>()
                    .join("\n");

                if fs::write(&out_path, &full_text).is_ok() {
                    on_done(Ok(CleanResult { 
                        lines_count: actual_log_count, 
                        preview, 
                        out_path: out_path.to_string_lossy().to_string() 
                    }));
                } else { 
                    on_done(Err("Ошибка записи в файл".to_string())); 
                }
            }
            Err(e) => on_done(Err(e)),
        }
    });
}