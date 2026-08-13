// secret_diary.cs — C# версия

using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

class Entry {
    public int Id { get; set; }
    public string Date { get; set; }
    public string Category { get; set; }
    public string Text { get; set; }
}

class SecretDiary {
    private List<Entry> entries = new List<Entry>();
    private string pinHash = null;
    private bool locked = true;
    private DateTime lastActivity = DateTime.Now;
    private const string DataFile = "secret_diary.json";
    private const int LockTimeout = 300; // 5 минут

    public SecretDiary() {
        Load();
    }

    private void Load() {
        if (File.Exists(DataFile)) {
            try {
                string json = File.ReadAllText(DataFile);
                var data = JsonSerializer.Deserialize<Dictionary<string, object>>(json);
                if (data != null) {
                    pinHash = data.ContainsKey("pinHash") ? data["pinHash"].ToString() : null;
                    var entriesData = data.ContainsKey("entries") ? data["entries"].ToString() : "[]";
                    entries = JsonSerializer.Deserialize<List<Entry>>(entriesData) ?? new List<Entry>();
                }
            } catch {
                pinHash = null;
                entries = new List<Entry>();
            }
        }
    }

    private void Save() {
        var data = new Dictionary<string, object> {
            ["pinHash"] = pinHash,
            ["entries"] = entries
        };
        string json = JsonSerializer.Serialize(data, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(DataFile, json);
    }

    private string HashPin(string pin) {
        using var sha256 = SHA256.Create();
        byte[] hash = sha256.ComputeHash(Encoding.UTF8.GetBytes(pin));
        return BitConverter.ToString(hash).Replace("-", "").ToLower();
    }

    private bool CheckPin(string pin) {
        if (string.IsNullOrEmpty(pinHash)) {
            pinHash = HashPin(pin);
            Save();
            return true;
        }
        return HashPin(pin) == pinHash;
    }

    public bool Unlock(string pin) {
        if (CheckPin(pin)) {
            locked = false;
            lastActivity = DateTime.Now;
            return true;
        }
        return false;
    }

    public bool CheckLock() {
        if ((DateTime.Now - lastActivity).TotalSeconds > LockTimeout) {
            locked = true;
            return true;
        }
        return false;
    }

    public bool ChangePin(string oldPin, string newPin) {
        if (CheckPin(oldPin)) {
            pinHash = HashPin(newPin);
            Save();
            return true;
        }
        return false;
    }

    public int AddEntry(string category, string text) {
        int id = entries.Count + 1;
        string date = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss");
        entries.Add(new Entry { Id = id, Date = date, Category = category, Text = text });
        Save();
        lastActivity = DateTime.Now;
        return id;
    }

    public void ListAll() {
        if (entries.Count == 0) {
            Console.WriteLine("\u001B[33mНет записей.\u001B[0m");
            return;
        }
        Console.WriteLine($"\u001B[36m{"ID",-4} {"Дата",-20} {"Категория",-12} {"Текст",-50}\u001B[0m");
        Console.WriteLine(new string('-', 90));
        foreach (var e in entries) {
            string text = e.Text.Length > 50 ? e.Text.Substring(0, 50) : e.Text;
            Console.WriteLine($"{e.Id,-4} {e.Date,-20} {e.Category,-12} {text,-50}");
        }
    }

    public void Search(string keyword) {
        var results = entries.FindAll(e => e.Text.ToLower().Contains(keyword.ToLower()));
        if (results.Count == 0) {
            Console.WriteLine("\u001B[33mНичего не найдено.\u001B[0m");
            return;
        }
        foreach (var e in results) {
            Console.WriteLine($"{e.Id}: {e.Date} | {e.Category} | {e.Text}");
        }
    }

    public bool Delete(int id) {
        var entry = entries.Find(e => e.Id == id);
        if (entry != null) {
            entries.Remove(entry);
            Save();
            lastActivity = DateTime.Now;
            return true;
        }
        return false;
    }

    public bool Edit(int id, string newText) {
        var entry = entries.Find(e => e.Id == id);
        if (entry != null) {
            entry.Text = newText;
            Save();
            lastActivity = DateTime.Now;
            return true;
        }
        return false;
    }

    public void ExportCSV(string filename = "diary_export.csv") {
        if (entries.Count == 0) {
            Console.WriteLine("\u001B[33mНет записей для экспорта.\u001B[0m");
            return;
        }
        using var writer = new StreamWriter(filename);
        writer.WriteLine("ID,Дата,Категория,Текст");
        foreach (var e in entries) {
            writer.WriteLine($"{e.Id},{e.Date},{e.Category},\"{e.Text}\"");
        }
        Console.WriteLine($"\u001B[32m💾 Экспорт CSV: {filename}\u001B[0m");
    }

    public void ExportJSON(string filename = "diary_export.json") {
        if (entries.Count == 0) {
            Console.WriteLine("\u001B[33mНет записей для экспорта.\u001B[0m");
            return;
        }
        string json = JsonSerializer.Serialize(entries, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(filename, json);
        Console.WriteLine($"\u001B[32m💾 Экспорт JSON: {filename}\u001B[0m");
    }

    private static bool IsDigit(string s) {
        foreach (char c in s) {
            if (!char.IsDigit(c)) return false;
        }
        return true;
    }

    public static void Main() {
        var diary = new SecretDiary();
        Console.WriteLine("\u001B[36m🔐 Secret Diary (C#)\u001B[0m");

        if (string.IsNullOrEmpty(diary.pinHash)) {
            Console.WriteLine("\u001B[33mПервый запуск! Установите PIN-код (4-6 цифр):\u001B[0m");
            while (true) {
                Console.Write("PIN: ");
                string pin = Console.ReadLine().Trim();
                if (IsDigit(pin) && pin.Length >= 4 && pin.Length <= 6) {
                    Console.Write("Повторите PIN: ");
                    string confirm = Console.ReadLine().Trim();
                    if (pin == confirm) {
                        diary.pinHash = diary.HashPin(pin);
                        diary.Save();
                        Console.WriteLine("\u001B[32m✅ PIN установлен!\u001B[0m");
                        diary.locked = false;
                        break;
                    } else {
                        Console.WriteLine("\u001B[31m❌ PIN не совпадают.\u001B[0m");
                    }
                } else {
                    Console.WriteLine("\u001B[31m❌ PIN должен быть 4-6 цифр.\u001B[0m");
                }
            }
        }

        int attempts = 3;
        while (diary.locked) {
            if (attempts <= 0) {
                Console.WriteLine("\u001B[31m❌ Слишком много неудачных попыток. Выход.\u001B[0m");
                Environment.Exit(1);
            }
            Console.Write("Введите PIN-код: ");
            string pin = Console.ReadLine().Trim();
            if (diary.Unlock(pin)) {
                Console.WriteLine("\u001B[32m✅ Добро пожаловать!\u001B[0m");
                break;
            } else {
                attempts--;
                Console.WriteLine($"\u001B[31m❌ Неверный PIN. Осталось попыток: {attempts}\u001B[0m");
            }
        }

        while (true) {
            if (diary.CheckLock()) {
                Console.WriteLine($"\n\u001B[33m🔒 Автоматическая блокировка (бездействие > {SecretDiary.LockTimeout} сек)\u001B[0m");
                attempts = 3;
                while (diary.locked) {
                    if (attempts <= 0) {
                        Console.WriteLine("\u001B[31m❌ Слишком много неудачных попыток. Выход.\u001B[0m");
                        Environment.Exit(1);
                    }
                    Console.Write("Введите PIN-код для разблокировки: ");
                    string pin = Console.ReadLine().Trim();
                    if (diary.Unlock(pin)) {
                        Console.WriteLine("\u001B[32m✅ Разблокировано!\u001B[0m");
                        break;
                    } else {
                        attempts--;
                        Console.WriteLine($"\u001B[31m❌ Неверный PIN. Осталось попыток: {attempts}\u001B[0m");
                    }
                }
            }

            Console.WriteLine("\n\u001B[36m🔐 Secret Diary (C#)\u001B[0m");
            Console.WriteLine("1. Добавить запись");
            Console.WriteLine("2. Показать все записи");
            Console.WriteLine("3. Поиск записей");
            Console.WriteLine("4. Редактировать запись");
            Console.WriteLine("5. Удалить запись");
            Console.WriteLine("6. Экспорт в CSV");
            Console.WriteLine("7. Экспорт в JSON");
            Console.WriteLine("8. Сменить PIN-код");
            Console.WriteLine("9. Выход");
            Console.Write("Выберите действие: ");
            string choice = Console.ReadLine().Trim();

            switch (choice) {
                case "1":
                    Console.Write("Категория (личное/работа/идеи/другое): ");
                    string category = Console.ReadLine().Trim().ToLower();
                    if (category != "личное" && category != "работа" && category != "идеи" && category != "другое") {
                        category = "другое";
                    }
                    Console.Write("Текст записи: ");
                    string text = Console.ReadLine().Trim();
                    int id = diary.AddEntry(category, text);
                    Console.WriteLine($"\u001B[32m✅ Запись добавлена (ID: {id})\u001B[0m");
                    break;
                case "2": diary.ListAll(); break;
                case "3":
                    Console.Write("Ключевое слово для поиска: ");
                    string keyword = Console.ReadLine().Trim();
                    diary.Search(keyword);
                    break;
                case "4":
                    diary.ListAll();
                    Console.Write("Введите ID для редактирования: ");
                    int editId = int.Parse(Console.ReadLine().Trim());
                    Console.Write("Новый текст: ");
                    string newText = Console.ReadLine().Trim();
                    if (diary.Edit(editId, newText)) {
                        Console.WriteLine("\u001B[32m✅ Запись обновлена.\u001B[0m");
                    } else {
                        Console.WriteLine("\u001B[31m❌ Запись не найдена.\u001B[0m");
                    }
                    break;
                case "5":
                    diary.ListAll();
                    Console.Write("Введите ID для удаления: ");
                    int delId = int.Parse(Console.ReadLine().Trim());
                    if (diary.Delete(delId)) {
                        Console.WriteLine("\u001B[32m✅ Запись удалена.\u001B[0m");
                    } else {
                        Console.WriteLine("\u001B[31m❌ Запись не найдена.\u001B[0m");
                    }
                    break;
                case "6": diary.ExportCSV(); break;
                case "7": diary.ExportJSON(); break;
                case "8":
                    Console.Write("Текущий PIN: ");
                    string old = Console.ReadLine().Trim();
                    Console.Write("Новый PIN (4-6 цифр): ");
                    string newPin = Console.ReadLine().Trim();
                    if (IsDigit(newPin) && newPin.Length >= 4 && newPin.Length <= 6) {
                        Console.Write("Повторите новый PIN: ");
                        string confirm = Console.ReadLine().Trim();
                        if (newPin == confirm) {
                            if (diary.ChangePin(old, newPin)) {
                                Console.WriteLine("\u001B[32m✅ PIN изменён.\u001B[0m");
                            } else {
                                Console.WriteLine("\u001B[31m❌ Неверный текущий PIN.\u001B[0m");
                            }
                        } else {
                            Console.WriteLine("\u001B[31m❌ PIN не совпадают.\u001B[0m");
                        }
                    } else {
                        Console.WriteLine("\u001B[31m❌ PIN должен быть 4-6 цифр.\u001B[0m");
                    }
                    break;
                case "9": Console.WriteLine("До свидания!"); return;
                default: Console.WriteLine("\u001B[31mНеверный выбор.\u001B[0m"); break;
            }
        }
    }
}
