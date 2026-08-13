// secret_diary.go — Go версия

package main

import (
	"bufio"
	"crypto/sha256"
	"encoding/csv"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

type Entry struct {
	ID       int    `json:"id"`
	Date     string `json:"date"`
	Category string `json:"category"`
	Text     string `json:"text"`
}

type Diary struct {
	PinHash   string  `json:"pin_hash"`
	Entries   []Entry `json:"entries"`
	Locked    bool
	LastActivity time.Time
	file      string
}

func NewDiary(file string) *Diary {
	d := &Diary{file: file, Locked: true, LastActivity: time.Now()}
	d.load()
	return d
}

func (d *Diary) load() {
	data, err := os.ReadFile(d.file)
	if err != nil {
		d.PinHash = ""
		d.Entries = []Entry{}
		return
	}
	json.Unmarshal(data, d)
}

func (d *Diary) save() {
	data, _ := json.MarshalIndent(d, "", "  ")
	os.WriteFile(d.file, data, 0644)
}

func (d *Diary) hashPin(pin string) string {
	hash := sha256.Sum256([]byte(pin))
	return hex.EncodeToString(hash[:])
}

func (d *Diary) checkPin(pin string) bool {
	if d.PinHash == "" {
		d.PinHash = d.hashPin(pin)
		d.save()
		return true
	}
	return d.hashPin(pin) == d.PinHash
}

func (d *Diary) changePin(oldPin, newPin string) bool {
	if d.checkPin(oldPin) {
		d.PinHash = d.hashPin(newPin)
		d.save()
		return true
	}
	return false
}

func (d *Diary) unlock(pin string) bool {
	if d.checkPin(pin) {
		d.Locked = false
		d.LastActivity = time.Now()
		return true
	}
	return false
}

func (d *Diary) checkLock() bool {
	if time.Since(d.LastActivity) > 300*time.Second {
		d.Locked = true
		return true
	}
	return false
}

func (d *Diary) addEntry(category, text string) int {
	id := len(d.Entries) + 1
	date := time.Now().Format("2006-01-02 15:04:05")
	d.Entries = append(d.Entries, Entry{ID: id, Date: date, Category: category, Text: text})
	d.save()
	d.LastActivity = time.Now()
	return id
}

func (d *Diary) listAll() {
	if len(d.Entries) == 0 {
		fmt.Println("\u001B[33mНет записей.\u001B[0m")
		return
	}
	fmt.Printf("\u001B[36m%-4s %-20s %-12s %-50s\u001B[0m\n", "ID", "Дата", "Категория", "Текст")
	fmt.Println(strings.Repeat("-", 90))
	for _, e := range d.Entries {
		text := e.Text
		if len(text) > 50 {
			text = text[:50]
		}
		fmt.Printf("%-4d %-20s %-12s %-50s\n", e.ID, e.Date, e.Category, text)
	}
}

func (d *Diary) search(keyword string) {
	results := []Entry{}
	for _, e := range d.Entries {
		if strings.Contains(strings.ToLower(e.Text), strings.ToLower(keyword)) {
			results = append(results, e)
		}
	}
	if len(results) == 0 {
		fmt.Println("\u001B[33mНичего не найдено.\u001B[0m")
		return
	}
	for _, e := range results {
		fmt.Printf("%d: %s | %s | %s\n", e.ID, e.Date, e.Category, e.Text)
	}
}

func (d *Diary) delete(id int) bool {
	for i, e := range d.Entries {
		if e.ID == id {
			d.Entries = append(d.Entries[:i], d.Entries[i+1:]...)
			d.save()
			d.LastActivity = time.Now()
			return true
		}
	}
	return false
}

func (d *Diary) edit(id int, newText string) bool {
	for i, e := range d.Entries {
		if e.ID == id {
			d.Entries[i].Text = newText
			d.save()
			d.LastActivity = time.Now()
			return true
		}
	}
	return false
}

func (d *Diary) exportCSV(filename string) {
	if len(d.Entries) == 0 {
		fmt.Println("\u001B[33mНет записей для экспорта.\u001B[0m")
		return
	}
	file, _ := os.Create(filename)
	defer file.Close()
	writer := csv.NewWriter(file)
	defer writer.Flush()
	writer.Write([]string{"ID", "Дата", "Категория", "Текст"})
	for _, e := range d.Entries {
		writer.Write([]string{strconv.Itoa(e.ID), e.Date, e.Category, e.Text})
	}
	fmt.Printf("\u001B[32m💾 Экспорт CSV: %s\u001B[0m\n", filename)
}

func (d *Diary) exportJSON(filename string) {
	if len(d.Entries) == 0 {
		fmt.Println("\u001B[33mНет записей для экспорта.\u001B[0m")
		return
	}
	data, _ := json.MarshalIndent(d.Entries, "", "  ")
	os.WriteFile(filename, data, 0644)
	fmt.Printf("\u001B[32m💾 Экспорт JSON: %s\u001B[0m\n", filename)
}

func main() {
	diary := NewDiary("secret_diary.json")
	reader := bufio.NewReader(os.Stdin)

	fmt.Println("\u001B[36m🔐 Secret Diary (Go)\u001B[0m")

	if diary.PinHash == "" {
		fmt.Println("\u001B[33mПервый запуск! Установите PIN-код (4-6 цифр):\u001B[0m")
		for {
			fmt.Print("PIN: ")
			pin, _ := reader.ReadString('\n')
			pin = strings.TrimSpace(pin)
			if len(pin) >= 4 && len(pin) <= 6 && isDigit(pin) {
				fmt.Print("Повторите PIN: ")
				confirm, _ := reader.ReadString('\n')
				confirm = strings.TrimSpace(confirm)
				if pin == confirm {
					diary.PinHash = diary.hashPin(pin)
					diary.save()
					fmt.Println("\u001B[32m✅ PIN установлен!\u001B[0m")
					diary.Locked = false
					break
				} else {
					fmt.Println("\u001B[31m❌ PIN не совпадают.\u001B[0m")
				}
			} else {
				fmt.Println("\u001B[31m❌ PIN должен быть 4-6 цифр.\u001B[0m")
			}
		}
	}

	attempts := 3
	for diary.Locked {
		if attempts <= 0 {
			fmt.Println("\u001B[31m❌ Слишком много неудачных попыток. Выход.\u001B[0m")
			os.Exit(1)
		}
		fmt.Print("Введите PIN-код: ")
		pin, _ := reader.ReadString('\n')
		pin = strings.TrimSpace(pin)
		if diary.unlock(pin) {
			fmt.Println("\u001B[32m✅ Добро пожаловать!\u001B[0m")
			break
		} else {
			attempts--
			fmt.Printf("\u001B[31m❌ Неверный PIN. Осталось попыток: %d\u001B[0m\n", attempts)
		}
	}

	for {
		if diary.checkLock() {
			fmt.Println("\n\u001B[33m🔒 Автоматическая блокировка (бездействие > 5 мин)\u001B[0m")
			attempts = 3
			for diary.Locked {
				if attempts <= 0 {
					fmt.Println("\u001B[31m❌ Слишком много неудачных попыток. Выход.\u001B[0m")
					os.Exit(1)
				}
				fmt.Print("Введите PIN-код для разблокировки: ")
				pin, _ := reader.ReadString('\n')
				pin = strings.TrimSpace(pin)
				if diary.unlock(pin) {
					fmt.Println("\u001B[32m✅ Разблокировано!\u001B[0m")
					break
				} else {
					attempts--
					fmt.Printf("\u001B[31m❌ Неверный PIN. Осталось попыток: %d\u001B[0m\n", attempts)
				}
			}
		}

		fmt.Println("\n\u001B[36m🔐 Secret Diary (Go)\u001B[0m")
		fmt.Println("1. Добавить запись")
		fmt.Println("2. Показать все записи")
		fmt.Println("3. Поиск записей")
		fmt.Println("4. Редактировать запись")
		fmt.Println("5. Удалить запись")
		fmt.Println("6. Экспорт в CSV")
		fmt.Println("7. Экспорт в JSON")
		fmt.Println("8. Сменить PIN-код")
		fmt.Println("9. Выход")
		fmt.Print("Выберите действие: ")
		choice, _ := reader.ReadString('\n')
		choice = strings.TrimSpace(choice)

		switch choice {
		case "1":
			fmt.Print("Категория (личное/работа/идеи/другое): ")
			category, _ := reader.ReadString('\n')
			category = strings.TrimSpace(strings.ToLower(category))
			if category != "личное" && category != "работа" && category != "идеи" && category != "другое" {
				category = "другое"
			}
			fmt.Print("Текст записи: ")
			text, _ := reader.ReadString('\n')
			text = strings.TrimSpace(text)
			id := diary.addEntry(category, text)
			fmt.Printf("\u001B[32m✅ Запись добавлена (ID: %d)\u001B[0m\n", id)
		case "2":
			diary.listAll()
		case "3":
			fmt.Print("Ключевое слово для поиска: ")
			keyword, _ := reader.ReadString('\n')
			keyword = strings.TrimSpace(keyword)
			diary.search(keyword)
		case "4":
			diary.listAll()
			fmt.Print("Введите ID для редактирования: ")
			idStr, _ := reader.ReadString('\n')
			id, _ := strconv.Atoi(strings.TrimSpace(idStr))
			fmt.Print("Новый текст: ")
			text, _ := reader.ReadString('\n')
			text = strings.TrimSpace(text)
			if diary.edit(id, text) {
				fmt.Println("\u001B[32m✅ Запись обновлена.\u001B[0m")
			} else {
				fmt.Println("\u001B[31m❌ Запись не найдена.\u001B[0m")
			}
		case "5":
			diary.listAll()
			fmt.Print("Введите ID для удаления: ")
			idStr, _ := reader.ReadString('\n')
			id, _ := strconv.Atoi(strings.TrimSpace(idStr))
			if diary.delete(id) {
				fmt.Println("\u001B[32m✅ Запись удалена.\u001B[0m")
			} else {
				fmt.Println("\u001B[31m❌ Запись не найдена.\u001B[0m")
			}
		case "6":
			fmt.Print("Имя CSV файла (по умолч. diary_export.csv): ")
			filename, _ := reader.ReadString('\n')
			filename = strings.TrimSpace(filename)
			if filename == "" {
				filename = "diary_export.csv"
			}
			diary.exportCSV(filename)
		case "7":
			fmt.Print("Имя JSON файла (по умолч. diary_export.json): ")
			filename, _ := reader.ReadString('\n')
			filename = strings.TrimSpace(filename)
			if filename == "" {
				filename = "diary_export.json"
			}
			diary.exportJSON(filename)
		case "8":
			fmt.Print("Текущий PIN: ")
			old, _ := reader.ReadString('\n')
			old = strings.TrimSpace(old)
			fmt.Print("Новый PIN (4-6 цифр): ")
			newPin, _ := reader.ReadString('\n')
			newPin = strings.TrimSpace(newPin)
			if len(newPin) >= 4 && len(newPin) <= 6 && isDigit(newPin) {
				fmt.Print("Повторите новый PIN: ")
				confirm, _ := reader.ReadString('\n')
				confirm = strings.TrimSpace(confirm)
				if newPin == confirm {
					if diary.changePin(old, newPin) {
						fmt.Println("\u001B[32m✅ PIN изменён.\u001B[0m")
					} else {
						fmt.Println("\u001B[31m❌ Неверный текущий PIN.\u001B[0m")
					}
				} else {
					fmt.Println("\u001B[31m❌ PIN не совпадают.\u001B[0m")
				}
			} else {
				fmt.Println("\u001B[31m❌ PIN должен быть 4-6 цифр.\u001B[0m")
			}
		case "9":
			fmt.Println("До свидания!")
			return
		default:
			fmt.Println("\u001B[31mНеверный выбор.\u001B[0m")
		}
	}
}

func isDigit(s string) bool {
	for _, c := range s {
		if c < '0' || c > '9' {
			return false
		}
	}
	return true
}
