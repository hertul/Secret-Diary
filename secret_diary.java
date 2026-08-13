// secret_diary.java — Java версия

import java.io.*;
import java.nio.file.*;
import java.security.*;
import java.time.*;
import java.util.*;

class Entry {
    int id;
    String date;
    String category;
    String text;

    Entry(int id, String date, String category, String text) {
        this.id = id;
        this.date = date;
        this.category = category;
        this.text = text;
    }

    String toJson() {
        return String.format("{\"id\":%d,\"date\":\"%s\",\"category\":\"%s\",\"text\":\"%s\"}",
                id, date, category, text);
    }
}

public class secret_diary {
    private static List<Entry> entries = new ArrayList<>();
    private static String pinHash = null;
    private static boolean locked = true;
    private static long lastActivity = System.currentTimeMillis();
    private static final String DATA_FILE = "secret_diary.json";
    private static final long LOCK_TIMEOUT = 300000; // 5 минут
    private static Scanner scanner = new Scanner(System.in);

    public static void main(String[] args) {
        load();
        System.out.println("\u001B[36m🔐 Secret Diary (Java)\u001B[0m");

        if (pinHash == null) {
            System.out.println("\u001B[33mПервый запуск! Установите PIN-код (4-6 цифр):\u001B[0m");
            while (true) {
                System.out.print("PIN: ");
                String pin = scanner.nextLine().trim();
                if (isDigit(pin) && pin.length() >= 4 && pin.length() <= 6) {
                    System.out.print("Повторите PIN: ");
                    String confirm = scanner.nextLine().trim();
                    if (pin.equals(confirm)) {
                        pinHash = hashPin(pin);
                        save();
                        System.out.println("\u001B[32m✅ PIN установлен!\u001B[0m");
                        locked = false;
                        break;
                    } else {
                        System.out.println("\u001B[31m❌ PIN не совпадают.\u001B[0m");
                    }
                } else {
                    System.out.println("\u001B[31m❌ PIN должен быть 4-6 цифр.\u001B[0m");
                }
            }
        }

        int attempts = 3;
        while (locked) {
            if (attempts <= 0) {
                System.out.println("\u001B[31m❌ Слишком много неудачных попыток. Выход.\u001B[0m");
                System.exit(1);
            }
            System.out.print("Введите PIN-код: ");
            String pin = scanner.nextLine().trim();
            if (unlock(pin)) {
                System.out.println("\u001B[32m✅ Добро пожаловать!\u001B[0m");
                break;
            } else {
                attempts--;
                System.out.printf("\u001B[31m❌ Неверный PIN. Осталось попыток: %d\u001B[0m\n", attempts);
            }
        }

        while (true) {
            if (checkLock()) {
                System.out.println("\n\u001B[33m🔒 Автоматическая блокировка (бездействие > 5 мин)\u001B[0m");
                attempts = 3;
                while (locked) {
                    if (attempts <= 0) {
                        System.out.println("\u001B[31m❌ Слишком много неудачных попыток. Выход.\u001B[0m");
                        System.exit(1);
                    }
                    System.out.print("Введите PIN-код для разблокировки: ");
                    String pin = scanner.nextLine().trim();
                    if (unlock(pin)) {
                        System.out.println("\u001B[32m✅ Разблокировано!\u001B[0m");
                        break;
                    } else {
                        attempts--;
                        System.out.printf("\u001B[31m❌ Неверный PIN. Осталось попыток: %d\u001B[0m\n", attempts);
                    }
                }
            }

            System.out.println("\n\u001B[36m🔐 Secret Diary (Java)\u001B[0m");
            System.out.println("1. Добавить запись");
            System.out.println("2. Показать все записи");
            System.out.println("3. Поиск записей");
            System.out.println("4. Редактировать запись");
            System.out.println("5. Удалить запись");
            System.out.println("6. Экспорт в CSV");
            System.out.println("7. Экспорт в JSON");
            System.out.println("8. Сменить PIN-код");
            System.out.println("9. Выход");
            System.out.print("Выберите действие: ");
            String choice = scanner.nextLine().trim();

            switch (choice) {
                case "1": addEntry(); break;
                case "2": listAll(); break;
                case "3": searchEntries(); break;
                case "4": editEntry(); break;
                case "5": deleteEntry(); break;
                case "6": exportCSV(); break;
                case "7": exportJSON(); break;
                case "8": changePin(); break;
                case "9": System.out.println("До свидания!"); return;
                default: System.out.println("\u001B[31mНеверный выбор.\u001B[0m");
            }
        }
    }

    private static void load() {
        try {
            String content = new String(Files.readAllBytes(Paths.get(DATA_FILE)));
            // Упрощённый парсинг
            entries = new ArrayList<>();
        } catch (IOException e) {
            entries = new ArrayList<>();
        }
    }

    private static void save() {
        try {
            StringBuilder sb = new StringBuilder("{");
            sb.append("\"pinHash\":\"").append(pinHash).append("\",");
            sb.append("\"entries\":[");
            for (int i = 0; i < entries.size(); i++) {
                sb.append(entries.get(i).toJson());
                if (i < entries.size() - 1) sb.append(",");
            }
            sb.append("]}");
            Files.write(Paths.get(DATA_FILE), sb.toString().getBytes());
        } catch (IOException e) {
            System.out.println("Ошибка сохранения.");
        }
    }

    private static String hashPin(String pin) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] hash = md.digest(pin.getBytes());
            StringBuilder sb = new StringBuilder();
            for (byte b : hash) {
                sb.append(String.format("%02x", b));
            }
            return sb.toString();
        } catch (NoSuchAlgorithmException e) {
            return "";
        }
    }

    private static boolean isDigit(String s) {
        return s.matches("\\d+");
    }

    private static boolean checkPin(String pin) {
        if (pinHash == null) {
            pinHash = hashPin(pin);
            save();
            return true;
        }
        return hashPin(pin).equals(pinHash);
    }

    private static boolean unlock(String pin) {
        if (checkPin(pin)) {
            locked = false;
            lastActivity = System.currentTimeMillis();
            return true;
        }
        return false;
    }

    private static boolean checkLock() {
        if (System.currentTimeMillis() - lastActivity > LOCK_TIMEOUT) {
            locked = true;
            return true;
        }
        return false;
    }

    private static boolean changePin(String oldPin, String newPin) {
        if (checkPin(oldPin)) {
            pinHash = hashPin(newPin);
            save();
            return true;
        }
        return false;
    }

    private static void addEntry() {
        System.out.print("Категория (личное/работа/идеи/другое): ");
        String category = scanner.nextLine().trim().toLowerCase();
        if (!category.equals("личное") && !category.equals("работа") && !category.equals("идеи") && !category.equals("другое")) {
            category = "другое";
        }
        System.out.print("Текст записи: ");
        String text = scanner.nextLine().trim();
        int id = entries.size() + 1;
        String date = LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss"));
        entries.add(new Entry(id, date, category, text));
        save();
        lastActivity = System.currentTimeMillis();
        System.out.println("\u001B[32m✅ Запись добавлена (ID: " + id + ")\u001B[0m");
    }

    private static void listAll() {
        if (entries.isEmpty()) {
            System.out.println("\u001B[33mНет записей.\u001B[0m");
            return;
        }
        System.out.printf("\u001B[36m%-4s %-20s %-12s %-50s\u001B[0m\n", "ID", "Дата", "Категория", "Текст");
        System.out.println("-".repeat(90));
        for (Entry e : entries) {
            String text = e.text.length() > 50 ? e.text.substring(0, 50) : e.text;
            System.out.printf("%-4d %-20s %-12s %-50s\n", e.id, e.date, e.category, text);
        }
    }

    private static void searchEntries() {
        System.out.print("Ключевое слово для поиска: ");
        String keyword = scanner.nextLine().trim().toLowerCase();
        List<Entry> results = new ArrayList<>();
        for (Entry e : entries) {
            if (e.text.toLowerCase().contains(keyword)) {
                results.add(e);
            }
        }
        if (results.isEmpty()) {
            System.out.println("\u001B[33mНичего не найдено.\u001B[0m");
            return;
        }
        for (Entry e : results) {
            System.out.printf("%d: %s | %s | %s\n", e.id, e.date, e.category, e.text);
        }
    }

    private static void editEntry() {
        listAll();
        System.out.print("Введите ID для редактирования: ");
        int id = Integer.parseInt(scanner.nextLine().trim());
        Entry target = null;
        for (Entry e : entries) {
            if (e.id == id) {
                target = e;
                break;
            }
        }
        if (target == null) {
            System.out.println("\u001B[31m❌ Запись не найдена.\u001B[0m");
            return;
        }
        System.out.print("Новый текст: ");
        String text = scanner.nextLine().trim();
        target.text = text;
        save();
        lastActivity = System.currentTimeMillis();
        System.out.println("\u001B[32m✅ Запись обновлена.\u001B[0m");
    }

    private static void deleteEntry() {
        listAll();
        System.out.print("Введите ID для удаления: ");
        int id = Integer.parseInt(scanner.nextLine().trim());
        Iterator<Entry> it = entries.iterator();
        while (it.hasNext()) {
            if (it.next().id == id) {
                it.remove();
                save();
                lastActivity = System.currentTimeMillis();
                System.out.println("\u001B[32m✅ Запись удалена.\u001B[0m");
                return;
            }
        }
        System.out.println("\u001B[31m❌ Запись не найдена.\u001B[0m");
    }

    private static void exportCSV() {
        if (entries.isEmpty()) {
            System.out.println("\u001B[33mНет записей для экспорта.\u001B[0m");
            return;
        }
        try (FileWriter fw = new FileWriter("diary_export.csv")) {
            fw.write("ID,Дата,Категория,Текст\n");
            for (Entry e : entries) {
                fw.write(e.id + "," + e.date + "," + e.category + ",\"" + e.text + "\"\n");
            }
            System.out.println("\u001B[32m💾 Экспорт CSV: diary_export.csv\u001B[0m");
        } catch (IOException e) {
            System.out.println("Ошибка экспорта.");
        }
    }

    private static void exportJSON() {
        if (entries.isEmpty()) {
            System.out.println("\u001B[33mНет записей для экспорта.\u001B[0m");
            return;
        }
        try {
            StringBuilder sb = new StringBuilder("[");
            for (int i = 0; i < entries.size(); i++) {
                sb.append(entries.get(i).toJson());
                if (i < entries.size() - 1) sb.append(",");
            }
            sb.append("]");
            Files.write(Paths.get("diary_export.json"), sb.toString().getBytes());
            System.out.println("\u001B[32m💾 Экспорт JSON: diary_export.json\u001B[0m");
        } catch (IOException e) {
            System.out.println("Ошибка экспорта.");
        }
    }

    private static void changePin() {
        System.out.print("Текущий PIN: ");
        String old = scanner.nextLine().trim();
        System.out.print("Новый PIN (4-6 цифр): ");
        String newPin = scanner.nextLine().trim();
        if (isDigit(newPin) && newPin.length() >= 4 && newPin.length() <= 6) {
            System.out.print("Повторите новый PIN: ");
            String confirm = scanner.nextLine().trim();
            if (newPin.equals(confirm)) {
                if (changePin(old, newPin)) {
                    System.out.println("\u001B[32m✅ PIN изменён.\u001B[0m");
                } else {
                    System.out.println("\u001B[31m❌ Неверный текущий PIN.\u001B[0m");
                }
            } else {
                System.out.println("\u001B[31m❌ PIN не совпадают.\u001B[0m");
            }
        } else {
            System.out.println("\u001B[31m❌ PIN должен быть 4-6 цифр.\u001B[0m");
        }
    }
}
