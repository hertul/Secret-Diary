

### 1. `secret_diary.py` (Python)

```python
# secret_diary.py — Python версия

import json
import os
import hashlib
import time
import sys
from datetime import datetime
from colorama import init, Fore, Style

init(autoreset=True)
DATA_FILE = "secret_diary.json"
LOCK_TIMEOUT = 300  # 5 минут

class Entry:
    def __init__(self, id, date, category, text):
        self.id = id
        self.date = date
        self.category = category
        self.text = text

    def to_dict(self):
        return {"id": self.id, "date": self.date, "category": self.category, "text": self.text}

    @classmethod
    def from_dict(cls, data):
        return cls(data["id"], data["date"], data["category"], data["text"])

class SecretDiary:
    def __init__(self):
        self.entries = []
        self.pin_hash = None
        self.locked = True
        self.last_activity = time.time()
        self.load()

    def load(self):
        if os.path.exists(DATA_FILE):
            try:
                with open(DATA_FILE, 'r', encoding='utf-8') as f:
                    data = json.load(f)
                    self.pin_hash = data.get("pin_hash")
                    self.entries = [Entry.from_dict(e) for e in data.get("entries", [])]
            except:
                self.pin_hash = None
                self.entries = []

    def save(self):
        data = {
            "pin_hash": self.pin_hash,
            "entries": [e.to_dict() for e in self.entries]
        }
        with open(DATA_FILE, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    def hash_pin(self, pin):
        return hashlib.sha256(pin.encode()).hexdigest()

    def check_pin(self, pin):
        if self.pin_hash is None:
            # Первый запуск: установка PIN
            self.pin_hash = self.hash_pin(pin)
            self.save()
            return True
        return self.hash_pin(pin) == self.pin_hash

    def change_pin(self, old_pin, new_pin):
        if self.check_pin(old_pin):
            self.pin_hash = self.hash_pin(new_pin)
            self.save()
            return True
        return False

    def check_lock(self):
        if time.time() - self.last_activity > LOCK_TIMEOUT:
            self.locked = True
            return True
        return False

    def unlock(self, pin):
        if self.check_pin(pin):
            self.locked = False
            self.last_activity = time.time()
            return True
        return False

    def add_entry(self, category, text):
        id = len(self.entries) + 1
        date = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        entry = Entry(id, date, category, text)
        self.entries.append(entry)
        self.save()
        self.last_activity = time.time()
        return id

    def list_all(self):
        if not self.entries:
            print(Fore.YELLOW + "Нет записей.")
            return
        print(Fore.CYAN + f"{'ID':<4} {'Дата':<20} {'Категория':<12} {'Текст':<50}")
        print("-" * 90)
        for e in self.entries:
            print(f"{e.id:<4} {e.date:<20} {e.category:<12} {e.text[:50]:<50}")

    def search(self, keyword):
        results = [e for e in self.entries if keyword.lower() in e.text.lower()]
        if not results:
            print(Fore.YELLOW + "Ничего не найдено.")
            return
        for e in results:
            print(f"{e.id}: {e.date} | {e.category} | {e.text[:50]}")

    def delete(self, id):
        for i, e in enumerate(self.entries):
            if e.id == id:
                del self.entries[i]
                self.save()
                self.last_activity = time.time()
                return True
        return False

    def edit(self, id, new_text):
        for e in self.entries:
            if e.id == id:
                e.text = new_text
                self.save()
                self.last_activity = time.time()
                return True
        return False

    def export_csv(self, filename="diary_export.csv"):
        if not self.entries:
            print(Fore.YELLOW + "Нет записей для экспорта.")
            return
        import csv
        with open(filename, 'w', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            writer.writerow(["ID", "Дата", "Категория", "Текст"])
            for e in self.entries:
                writer.writerow([e.id, e.date, e.category, e.text])
        print(Fore.GREEN + f"💾 Экспорт CSV: {filename}")

    def export_json(self, filename="diary_export.json"):
        if not self.entries:
            print(Fore.YELLOW + "Нет записей для экспорта.")
            return
        data = [e.to_dict() for e in self.entries]
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        print(Fore.GREEN + f"💾 Экспорт JSON: {filename}")

def main():
    diary = SecretDiary()

    print(Fore.CYAN + "🔐 Secret Diary (Python)")

    # Первый запуск или ввод PIN
    if diary.pin_hash is None:
        print(Fore.YELLOW + "Первый запуск! Установите PIN-код (4-6 цифр):")
        while True:
            pin = input("PIN: ").strip()
            if pin.isdigit() and 4 <= len(pin) <= 6:
                confirm = input("Повторите PIN: ").strip()
                if pin == confirm:
                    diary.pin_hash = diary.hash_pin(pin)
                    diary.save()
                    print(Fore.GREEN + "✅ PIN установлен!")
                    diary.locked = False
                    break
                else:
                    print(Fore.RED + "❌ PIN не совпадают.")
            else:
                print(Fore.RED + "❌ PIN должен быть 4-6 цифр.")

    # Вход
    attempts = 3
    while diary.locked:
        if attempts <= 0:
            print(Fore.RED + "❌ Слишком много неудачных попыток. Выход.")
            sys.exit(1)
        pin = input("Введите PIN-код: ").strip()
        if diary.unlock(pin):
            print(Fore.GREEN + "✅ Добро пожаловать!")
            break
        else:
            attempts -= 1
            print(Fore.RED + f"❌ Неверный PIN. Осталось попыток: {attempts}")

    while True:
        if diary.check_lock():
            print(Fore.YELLOW + "\n🔒 Автоматическая блокировка (бездействие > 5 мин)")
            attempts = 3
            while diary.locked:
                if attempts <= 0:
                    print(Fore.RED + "❌ Слишком много неудачных попыток. Выход.")
                    sys.exit(1)
                pin = input("Введите PIN-код для разблокировки: ").strip()
                if diary.unlock(pin):
                    print(Fore.GREEN + "✅ Разблокировано!")
                    break
                else:
                    attempts -= 1
                    print(Fore.RED + f"❌ Неверный PIN. Осталось попыток: {attempts}")

        print(Fore.CYAN + "\n🔐 Secret Diary")
        print("1. Добавить запись")
        print("2. Показать все записи")
        print("3. Поиск записей")
        print("4. Редактировать запись")
        print("5. Удалить запись")
        print("6. Экспорт в CSV")
        print("7. Экспорт в JSON")
        print("8. Сменить PIN-код")
        print("9. Выход")
        choice = input("Выберите действие: ").strip()

        if choice == "1":
            category = input("Категория (личное/работа/идеи/другое): ").strip().lower()
            if category not in ["личное", "работа", "идеи", "другое"]:
                category = "другое"
            text = input("Текст записи: ")
            id = diary.add_entry(category, text)
            print(Fore.GREEN + f"✅ Запись добавлена (ID: {id})")
        elif choice == "2":
            diary.list_all()
        elif choice == "3":
            keyword = input("Ключевое слово для поиска: ")
            diary.search(keyword)
        elif choice == "4":
            diary.list_all()
            id = int(input("Введите ID для редактирования: "))
            new_text = input("Новый текст: ")
            if diary.edit(id, new_text):
                print(Fore.GREEN + "✅ Запись обновлена.")
            else:
                print(Fore.RED + "❌ Запись не найдена.")
        elif choice == "5":
            diary.list_all()
            id = int(input("Введите ID для удаления: "))
            if diary.delete(id):
                print(Fore.GREEN + "✅ Запись удалена.")
            else:
                print(Fore.RED + "❌ Запись не найдена.")
        elif choice == "6":
            filename = input("Имя CSV файла (по умолч. diary_export.csv): ").strip()
            if not filename:
                filename = "diary_export.csv"
            diary.export_csv(filename)
        elif choice == "7":
            filename = input("Имя JSON файла (по умолч. diary_export.json): ").strip()
            if not filename:
                filename = "diary_export.json"
            diary.export_json(filename)
        elif choice == "8":
            old = input("Текущий PIN: ").strip()
            new = input("Новый PIN (4-6 цифр): ").strip()
            if new.isdigit() and 4 <= len(new) <= 6:
                confirm = input("Повторите новый PIN: ").strip()
                if new == confirm:
                    if diary.change_pin(old, new):
                        print(Fore.GREEN + "✅ PIN изменён.")
                    else:
                        print(Fore.RED + "❌ Неверный текущий PIN.")
                else:
                    print(Fore.RED + "❌ PIN не совпадают.")
            else:
                print(Fore.RED + "❌ PIN должен быть 4-6 цифр.")
        elif choice == "9":
            print("До свидания!")
            break
        else:
            print(Fore.RED + "Неверный выбор.")

if __name__ == "__main__":
    main()
