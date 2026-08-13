// secret_diary.rs — Rust версия

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use sha2::{Sha256, Digest};
use chrono::Local;

#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    id: usize,
    date: String,
    category: String,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct DiaryData {
    pin_hash: String,
    entries: Vec<Entry>,
}

struct SecretDiary {
    entries: Vec<Entry>,
    pin_hash: Option<String>,
    locked: bool,
    last_activity: Instant,
    file: String,
}

impl SecretDiary {
    fn new(file: &str) -> Self {
        let mut d = SecretDiary {
            entries: Vec::new(),
            pin_hash: None,
            locked: true,
            last_activity: Instant::now(),
            file: file.to_string(),
        };
        d.load();
        d
    }

    fn load(&mut self) {
        if let Ok(data) = fs::read_to_string(&self.file) {
            if let Ok(diary_data) = serde_json::from_str::<DiaryData>(&data) {
                self.pin_hash = Some(diary_data.pin_hash);
                self.entries = diary_data.entries;
                return;
            }
        }
        self.pin_hash = None;
        self.entries = Vec::new();
    }

    fn save(&self) {
        let data = DiaryData {
            pin_hash: self.pin_hash.clone().unwrap_or_default(),
            entries: self.entries.clone(),
        };
        let json = serde_json::to_string_pretty(&data).unwrap();
        fs::write(&self.file, json).unwrap();
    }

    fn hash_pin(pin: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn check_pin(&mut self, pin: &str) -> bool {
        if let Some(ref stored_hash) = self.pin_hash {
            *stored_hash == Self::hash_pin(pin)
        } else {
            self.pin_hash = Some(Self::hash_pin(pin));
            self.save();
            true
        }
    }

    fn unlock(&mut self, pin: &str) -> bool {
        if self.check_pin(pin) {
            self.locked = false;
            self.last_activity = Instant::now();
            true
        } else {
            false
        }
    }

    fn check_lock(&mut self) -> bool {
        if self.last_activity.elapsed() > Duration::from_secs(300) {
            self.locked = true;
            true
        } else {
            false
        }
    }

    fn change_pin(&mut self, old_pin: &str, new_pin: &str) -> bool {
        if self.check_pin(old_pin) {
            self.pin_hash = Some(Self::hash_pin(new_pin));
            self.save();
            true
        } else {
            false
        }
    }

    fn add_entry(&mut self, category: String, text: String) -> usize {
        let id = self.entries.len() + 1;
        let date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.entries.push(Entry { id, date, category, text });
        self.save();
        self.last_activity = Instant::now();
        id
    }

    fn list_all(&self) {
        if self.entries.is_empty() {
            println!("\x1b[33mНет записей.\x1b[0m");
            return;
        }
        println!("\x1b[36m{:<4} {:<20} {:<12} {:<50}\x1b[0m", "ID", "Дата", "Категория", "Текст");
        println!("{}", "-".repeat(90));
        for e in &self.entries {
            let text = if e.text.len() > 50 { &e.text[..50] } else { &e.text };
            println!("{:<4} {:<20} {:<12} {:<50}", e.id, e.date, e.category, text);
        }
    }

    fn search(&self, keyword: &str) {
        let results: Vec<&Entry> = self.entries.iter()
            .filter(|e| e.text.to_lowercase().contains(&keyword.to_lowercase()))
            .collect();
        if results.is_empty() {
            println!("\x1b[33mНичего не найдено.\x1b[0m");
            return;
        }
        for e in results {
            println!("{}: {} | {} | {}", e.id, e.date, e.category, e.text);
        }
    }

    fn delete(&mut self, id: usize) -> bool {
        let pos = self.entries.iter().position(|e| e.id == id);
        if let Some(idx) = pos {
            self.entries.remove(idx);
            self.save();
            self.last_activity = Instant::now();
            true
        } else {
            false
        }
    }

    fn edit(&mut self, id: usize, new_text: &str) -> bool {
        for e in &mut self.entries {
            if e.id == id {
                e.text = new_text.to_string();
                self.save();
                self.last_activity = Instant::now();
                return true;
            }
        }
        false
    }

    fn export_csv(&self, filename: &str) {
        if self.entries.is_empty() {
            println!("\x1b[33mНет записей для экспорта.\x1b[0m");
            return;
        }
        let mut csv = String::from("ID,Дата,Категория,Текст\n");
        for e in &self.entries {
            csv.push_str(&format!("{},{},{},\"{}\"\n", e.id, e.date, e.category, e.text));
        }
        fs::write(filename, csv).unwrap();
        println!("\x1b[32m💾 Экспорт CSV: {}\x1b[0m", filename);
    }

    fn export_json(&self, filename: &str) {
        if self.entries.is_empty() {
            println!("\x1b[33mНет записей для экспорта.\x1b[0m");
            return;
        }
        let json = serde_json::to_string_pretty(&self.entries).unwrap();
        fs::write(filename, json).unwrap();
        println!("\x1b[32m💾 Экспорт JSON: {}\x1b[0m", filename);
    }
}

fn is_digit(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

fn main() {
    let mut diary = SecretDiary::new("secret_diary.json");
    println!("\x1b[36m🔐 Secret Diary (Rust)\x1b[0m");

    if diary.pin_hash.is_none() {
        println!("\x1b[33mПервый запуск! Установите PIN-код (4-6 цифр):\x1b[0m");
        loop {
            print!("PIN: ");
            io::stdout().flush().unwrap();
            let mut pin = String::new();
            io::stdin().read_line(&mut pin).unwrap();
            let pin = pin.trim();
            if is_digit(pin) && pin.len() >= 4 && pin.len() <= 6 {
                print!("Повторите PIN: ");
                io::stdout().flush().unwrap();
                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm).unwrap();
                let confirm = confirm.trim();
                if pin == confirm {
                    diary.pin_hash = Some(SecretDiary::hash_pin(pin));
                    diary.save();
                    println!("\x1b[32m✅ PIN установлен!\x1b[0m");
                    diary.locked = false;
                    break;
                } else {
                    println!("\x1b[31m❌ PIN не совпадают.\x1b[0m");
                }
            } else {
                println!("\x1b[31m❌ PIN должен быть 4-6 цифр.\x1b[0m");
            }
        }
    }

    let mut attempts = 3;
    while diary.locked {
        if attempts <= 0 {
            println!("\x1b[31m❌ Слишком много неудачных попыток. Выход.\x1b[0m");
            std::process::exit(1);
        }
        print!("Введите PIN-код: ");
        io::stdout().flush().unwrap();
        let mut pin = String::new();
        io::stdin().read_line(&mut pin).unwrap();
        let pin = pin.trim();
        if diary.unlock(pin) {
            println!("\x1b[32m✅ Добро пожаловать!\x1b[0m");
            break;
        } else {
            attempts -= 1;
            println!("\x1b[31m❌ Неверный PIN. Осталось попыток: {}\x1b[0m", attempts);
        }
    }

    loop {
        if diary.check_lock() {
            println!("\n\x1b[33m🔒 Автоматическая блокировка (бездействие > 5 мин)\x1b[0m");
            attempts = 3;
            while diary.locked {
                if attempts <= 0 {
                    println!("\x1b[31m❌ Слишком много неудачных попыток. Выход.\x1b[0m");
                    std::process::exit(1);
                }
                print!("Введите PIN-код для разблокировки: ");
                io::stdout().flush().unwrap();
                let mut pin = String::new();
                io::stdin().read_line(&mut pin).unwrap();
                let pin = pin.trim();
                if diary.unlock(pin) {
                    println!("\x1b[32m✅ Разблокировано!\x1b[0m");
                    break;
                } else {
                    attempts -= 1;
                    println!("\x1b[31m❌ Неверный PIN. Осталось попыток: {}\x1b[0m", attempts);
                }
            }
        }

        println!("\n\x1b[36m🔐 Secret Diary (Rust)\x1b[0m");
        println!("1. Добавить запись");
        println!("2. Показать все записи");
        println!("3. Поиск записей");
        println!("4. Редактировать запись");
        println!("5. Удалить запись");
        println!("6. Экспорт в CSV");
        println!("7. Экспорт в JSON");
        println!("8. Сменить PIN-код");
        println!("9. Выход");
        print!("Выберите действие: ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                print!("Категория (личное/работа/идеи/другое): ");
                io::stdout().flush().unwrap();
                let mut category = String::new();
                io::stdin().read_line(&mut category).unwrap();
                let category = category.trim().to_lowercase();
                let category = if category == "личное" || category == "работа" || category == "идеи" || category == "другое" {
                    category
                } else {
                    "другое".to_string()
                };
                print!("Текст записи: ");
                io::stdout().flush().unwrap();
                let mut text = String::new();
                io::stdin().read_line(&mut text).unwrap();
                let text = text.trim().to_string();
                let id = diary.add_entry(category, text);
                println!("\x1b[32m✅ Запись добавлена (ID: {})\x1b[0m", id);
            }
            "2" => diary.list_all(),
            "3" => {
                print!("Ключевое слово для поиска: ");
                io::stdout().flush().unwrap();
                let mut keyword = String::new();
                io::stdin().read_line(&mut keyword).unwrap();
                diary.search(keyword.trim());
            }
            "4" => {
                diary.list_all();
                print!("Введите ID для редактирования: ");
                io::stdout().flush().unwrap();
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str).unwrap();
                let id: usize = id_str.trim().parse().unwrap();
                print!("Новый текст: ");
                io::stdout().flush().unwrap();
                let mut text = String::new();
                io::stdin().read_line(&mut text).unwrap();
                if diary.edit(id, text.trim()) {
                    println!("\x1b[32m✅ Запись обновлена.\x1b[0m");
                } else {
                    println!("\x1b[31m❌ Запись не найдена.\x1b[0m");
                }
            }
            "5" => {
                diary.list_all();
                print!("Введите ID для удаления: ");
                io::stdout().flush().unwrap();
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str).unwrap();
                let id: usize = id_str.trim().parse().unwrap();
                if diary.delete(id) {
                    println!("\x1b[32m✅ Запись удалена.\x1b[0m");
                } else {
                    println!("\x1b[31m❌ Запись не найдена.\x1b[0m");
                }
            }
            "6" => {
                print!("Имя CSV файла (по умолч. diary_export.csv): ");
                io::stdout().flush().unwrap();
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).unwrap();
                let filename = if filename.trim().is_empty() { "diary_export.csv" } else { filename.trim() };
                diary.export_csv(filename);
            }
            "7" => {
                print!("Имя JSON файла (по умолч. diary_export.json): ");
                io::stdout().flush().unwrap();
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).unwrap();
                let filename = if filename.trim().is_empty() { "diary_export.json" } else { filename.trim() };
                diary.export_json(filename);
            }
            "8" => {
                print!("Текущий PIN: ");
                io::stdout().flush().unwrap();
                let mut old = String::new();
                io::stdin().read_line(&mut old).unwrap();
                let old = old.trim();
                print!("Новый PIN (4-6 цифр): ");
                io::stdout().flush().unwrap();
                let mut new_pin = String::new();
                io::stdin().read_line(&mut new_pin).unwrap();
                let new_pin = new_pin.trim();
                if is_digit(new_pin) && new_pin.len() >= 4 && new_pin.len() <= 6 {
                    print!("Повторите новый PIN: ");
                    io::stdout().flush().unwrap();
                    let mut confirm = String::new();
                    io::stdin().read_line(&mut confirm).unwrap();
                    let confirm = confirm.trim();
                    if new_pin == confirm {
                        if diary.change_pin(old, new_pin) {
                            println!("\x1b[32m✅ PIN изменён.\x1b[0m");
                        } else {
                            println!("\x1b[31m❌ Неверный текущий PIN.\x1b[0m");
                        }
                    } else {
                        println!("\x1b[31m❌ PIN не совпадают.\x1b[0m");
                    }
                } else {
                    println!("\x1b[31m❌ PIN должен быть 4-6 цифр.\x1b[0m");
                }
            }
            "9" => {
                println!("До свидания!");
                break;
            }
            _ => println!("\x1b[31mНеверный выбор.\x1b[0m"),
        }
    }
}
