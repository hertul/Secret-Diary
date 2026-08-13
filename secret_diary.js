// secret_diary.js — JavaScript версия

const fs = require('fs');
const readline = require('readline');
const crypto = require('crypto');

const DATA_FILE = 'secret_diary.json';
const LOCK_TIMEOUT = 300000; // 5 минут

class Entry {
    constructor(id, date, category, text) {
        this.id = id;
        this.date = date;
        this.category = category;
        this.text = text;
    }
}

class SecretDiary {
    constructor() {
        this.entries = [];
        this.pinHash = null;
        this.locked = true;
        this.lastActivity = Date.now();
        this.load();
    }

    load() {
        if (fs.existsSync(DATA_FILE)) {
            try {
                const data = JSON.parse(fs.readFileSync(DATA_FILE, 'utf8'));
                this.pinHash = data.pinHash || null;
                this.entries = (data.entries || []).map(e => new Entry(e.id, e.date, e.category, e.text));
            } catch {
                this.pinHash = null;
                this.entries = [];
            }
        }
    }

    save() {
        const data = {
            pinHash: this.pinHash,
            entries: this.entries
        };
        fs.writeFileSync(DATA_FILE, JSON.stringify(data, null, 2));
    }

    hashPin(pin) {
        return crypto.createHash('sha256').update(pin).digest('hex');
    }

    checkPin(pin) {
        if (!this.pinHash) {
            this.pinHash = this.hashPin(pin);
            this.save();
            return true;
        }
        return this.hashPin(pin) === this.pinHash;
    }

    changePin(oldPin, newPin) {
        if (this.checkPin(oldPin)) {
            this.pinHash = this.hashPin(newPin);
            this.save();
            return true;
        }
        return false;
    }

    checkLock() {
        if (Date.now() - this.lastActivity > LOCK_TIMEOUT) {
            this.locked = true;
            return true;
        }
        return false;
    }

    unlock(pin) {
        if (this.checkPin(pin)) {
            this.locked = false;
            this.lastActivity = Date.now();
            return true;
        }
        return false;
    }

    addEntry(category, text) {
        const id = this.entries.length + 1;
        const date = new Date().toISOString().replace('T', ' ').slice(0, 19);
        const entry = new Entry(id, date, category, text);
        this.entries.push(entry);
        this.save();
        this.lastActivity = Date.now();
        return id;
    }

    listAll() {
        if (this.entries.length === 0) {
            console.log('\x1b[33mНет записей.\x1b[0m');
            return;
        }
        console.log('\x1b[36m' + 'ID'.padEnd(4) + 'Дата'.padEnd(20) + 'Категория'.padEnd(12) + 'Текст'.padEnd(50) + '\x1b[0m');
        console.log('-'.repeat(90));
        for (const e of this.entries) {
            const text = e.text.length > 50 ? e.text.slice(0, 50) : e.text;
            console.log(`${String(e.id).padEnd(4)} ${e.date.padEnd(20)} ${e.category.padEnd(12)} ${text.padEnd(50)}`);
        }
    }

    search(keyword) {
        const results = this.entries.filter(e => e.text.toLowerCase().includes(keyword.toLowerCase()));
        if (results.length === 0) {
            console.log('\x1b[33mНичего не найдено.\x1b[0m');
            return;
        }
        for (const e of results) {
            console.log(`${e.id}: ${e.date} | ${e.category} | ${e.text}`);
        }
    }

    delete(id) {
        const index = this.entries.findIndex(e => e.id === id);
        if (index !== -1) {
            this.entries.splice(index, 1);
            this.save();
            this.lastActivity = Date.now();
            return true;
        }
        return false;
    }

    edit(id, newText) {
        const entry = this.entries.find(e => e.id === id);
        if (entry) {
            entry.text = newText;
            this.save();
            this.lastActivity = Date.now();
            return true;
        }
        return false;
    }

    exportCSV(filename = 'diary_export.csv') {
        if (this.entries.length === 0) {
            console.log('\x1b[33mНет записей для экспорта.\x1b[0m');
            return;
        }
        let csv = 'ID,Дата,Категория,Текст\n';
        for (const e of this.entries) {
            csv += `${e.id},${e.date},${e.category},"${e.text}"\n`;
        }
        fs.writeFileSync(filename, csv);
        console.log(`\x1b[32m💾 Экспорт CSV: ${filename}\x1b[0m`);
    }

    exportJSON(filename = 'diary_export.json') {
        if (this.entries.length === 0) {
            console.log('\x1b[33mНет записей для экспорта.\x1b[0m');
            return;
        }
        fs.writeFileSync(filename, JSON.stringify(this.entries, null, 2));
        console.log(`\x1b[32m💾 Экспорт JSON: ${filename}\x1b[0m`);
    }
}

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

const diary = new SecretDiary();

function ask(question) {
    return new Promise(resolve => rl.question(question, resolve));
}

function isDigit(s) {
    return /^\d+$/.test(s);
}

async function main() {
    console.log('\x1b[36m🔐 Secret Diary (JavaScript)\x1b[0m');

    if (!diary.pinHash) {
        console.log('\x1b[33mПервый запуск! Установите PIN-код (4-6 цифр):\x1b[0m');
        while (true) {
            const pin = await ask('PIN: ');
            if (isDigit(pin) && pin.length >= 4 && pin.length <= 6) {
                const confirm = await ask('Повторите PIN: ');
                if (pin === confirm) {
                    diary.pinHash = diary.hashPin(pin);
                    diary.save();
                    console.log('\x1b[32m✅ PIN установлен!\x1b[0m');
                    diary.locked = false;
                    break;
                } else {
                    console.log('\x1b[31m❌ PIN не совпадают.\x1b[0m');
                }
            } else {
                console.log('\x1b[31m❌ PIN должен быть 4-6 цифр.\x1b[0m');
            }
        }
    }

    let attempts = 3;
    while (diary.locked) {
        if (attempts <= 0) {
            console.log('\x1b[31m❌ Слишком много неудачных попыток. Выход.\x1b[0m');
            process.exit(1);
        }
        const pin = await ask('Введите PIN-код: ');
        if (diary.unlock(pin)) {
            console.log('\x1b[32m✅ Добро пожаловать!\x1b[0m');
            break;
        } else {
            attempts--;
            console.log(`\x1b[31m❌ Неверный PIN. Осталось попыток: ${attempts}\x1b[0m`);
        }
    }

    while (true) {
        if (diary.checkLock()) {
            console.log('\n\x1b[33m🔒 Автоматическая блокировка (бездействие > 5 мин)\x1b[0m');
            attempts = 3;
            while (diary.locked) {
                if (attempts <= 0) {
                    console.log('\x1b[31m❌ Слишком много неудачных попыток. Выход.\x1b[0m');
                    process.exit(1);
                }
                const pin = await ask('Введите PIN-код для разблокировки: ');
                if (diary.unlock(pin)) {
                    console.log('\x1b[32m✅ Разблокировано!\x1b[0m');
                    break;
                } else {
                    attempts--;
                    console.log(`\x1b[31m❌ Неверный PIN. Осталось попыток: ${attempts}\x1b[0m`);
                }
            }
        }

        console.log('\n\x1b[36m🔐 Secret Diary (JavaScript)\x1b[0m');
        console.log('1. Добавить запись');
        console.log('2. Показать все записи');
        console.log('3. Поиск записей');
        console.log('4. Редактировать запись');
        console.log('5. Удалить запись');
        console.log('6. Экспорт в CSV');
        console.log('7. Экспорт в JSON');
        console.log('8. Сменить PIN-код');
        console.log('9. Выход');
        const choice = await ask('Выберите действие: ');

        switch (choice.trim()) {
            case '1': {
                let category = await ask('Категория (личное/работа/идеи/другое): ');
                category = category.trim().toLowerCase();
                if (!['личное', 'работа', 'идеи', 'другое'].includes(category)) category = 'другое';
                const text = await ask('Текст записи: ');
                const id = diary.addEntry(category, text);
                console.log(`\x1b[32m✅ Запись добавлена (ID: ${id})\x1b[0m`);
                break;
            }
            case '2': diary.listAll(); break;
            case '3': {
                const keyword = await ask('Ключевое слово для поиска: ');
                diary.search(keyword);
                break;
            }
            case '4': {
                diary.listAll();
                const id = parseInt(await ask('Введите ID для редактирования: '));
                const newText = await ask('Новый текст: ');
                if (diary.edit(id, newText)) {
                    console.log('\x1b[32m✅ Запись обновлена.\x1b[0m');
                } else {
                    console.log('\x1b[31m❌ Запись не найдена.\x1b[0m');
                }
                break;
            }
            case '5': {
                diary.listAll();
                const id = parseInt(await ask('Введите ID для удаления: '));
                if (diary.delete(id)) {
                    console.log('\x1b[32m✅ Запись удалена.\x1b[0m');
                } else {
                    console.log('\x1b[31m❌ Запись не найдена.\x1b[0m');
                }
                break;
            }
            case '6': {
                let filename = await ask('Имя CSV файла (по умолч. diary_export.csv): ');
                if (!filename.trim()) filename = 'diary_export.csv';
                diary.exportCSV(filename);
                break;
            }
            case '7': {
                let filename = await ask('Имя JSON файла (по умолч. diary_export.json): ');
                if (!filename.trim()) filename = 'diary_export.json';
                diary.exportJSON(filename);
                break;
            }
            case '8': {
                const old = await ask('Текущий PIN: ');
                const newPin = await ask('Новый PIN (4-6 цифр): ');
                if (isDigit(newPin) && newPin.length >= 4 && newPin.length <= 6) {
                    const confirm = await ask('Повторите новый PIN: ');
                    if (newPin === confirm) {
                        if (diary.changePin(old, newPin)) {
                            console.log('\x1b[32m✅ PIN изменён.\x1b[0m');
                        } else {
                            console.log('\x1b[31m❌ Неверный текущий PIN.\x1b[0m');
                        }
                    } else {
                        console.log('\x1b[31m❌ PIN не совпадают.\x1b[0m');
                    }
                } else {
                    console.log('\x1b[31m❌ PIN должен быть 4-6 цифр.\x1b[0m');
                }
                break;
            }
            case '9':
                console.log('До свидания!');
                rl.close();
                return;
            default:
                console.log('\x1b[31mНеверный выбор.\x1b[0m');
        }
    }
}

main().catch(console.error);
