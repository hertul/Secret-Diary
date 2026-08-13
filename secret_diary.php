<?php
// secret_diary.php — PHP версия

$dataFile = 'secret_diary.json';
define('LOCK_TIMEOUT', 300);

function loadDiary() {
    global $dataFile;
    if (file_exists($dataFile)) {
        $json = file_get_contents($dataFile);
        $data = json_decode($json, true);
        if ($data) {
            return $data;
        }
    }
    return ['pin_hash' => null, 'entries' => []];
}

function saveDiary($data) {
    global $dataFile;
    file_put_contents($dataFile, json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE));
}

function hashPin($pin) {
    return hash('sha256', $pin);
}

function isDigit($s) {
    return ctype_digit($s);
}

function color($text, $code) {
    return "\033[{$code}m{$text}\033[0m";
}

$diary = loadDiary();
$locked = true;
$lastActivity = time();

echo color("🔐 Secret Diary (PHP)\n", '36');

if (is_null($diary['pin_hash'])) {
    echo color("Первый запуск! Установите PIN-код (4-6 цифр):\n", '33');
    while (true) {
        echo "PIN: ";
        $pin = trim(fgets(STDIN));
        if (isDigit($pin) && strlen($pin) >= 4 && strlen($pin) <= 6) {
            echo "Повторите PIN: ";
            $confirm = trim(fgets(STDIN));
            if ($pin == $confirm) {
                $diary['pin_hash'] = hashPin($pin);
                saveDiary($diary);
                echo color("✅ PIN установлен!\n", '32');
                $locked = false;
                break;
            } else {
                echo color("❌ PIN не совпадают.\n", '31');
            }
        } else {
            echo color("❌ PIN должен быть 4-6 цифр.\n", '31');
        }
    }
}

$attempts = 3;
while ($locked) {
    if ($attempts <= 0) {
        echo color("❌ Слишком много неудачных попыток. Выход.\n", '31');
        exit(1);
    }
    echo "Введите PIN-код: ";
    $pin = trim(fgets(STDIN));
    if (hashPin($pin) == $diary['pin_hash']) {
        echo color("✅ Добро пожаловать!\n", '32');
        $locked = false;
        $lastActivity = time();
        break;
    } else {
        $attempts--;
        echo color("❌ Неверный PIN. Осталось попыток: $attempts\n", '31');
    }
}

function checkLock(&$locked, &$lastActivity) {
    if (time() - $lastActivity > LOCK_TIMEOUT) {
        $locked = true;
        return true;
    }
    return false;
}

while (true) {
    if (checkLock($locked, $lastActivity)) {
        echo color("\n🔒 Автоматическая блокировка (бездействие > 5 мин)\n", '33');
        $attempts = 3;
        while ($locked) {
            if ($attempts <= 0) {
                echo color("❌ Слишком много неудачных попыток. Выход.\n", '31');
                exit(1);
            }
            echo "Введите PIN-код для разблокировки: ";
            $pin = trim(fgets(STDIN));
            if (hashPin($pin) == $diary['pin_hash']) {
                echo color("✅ Разблокировано!\n", '32');
                $locked = false;
                $lastActivity = time();
                break;
            } else {
                $attempts--;
                echo color("❌ Неверный PIN. Осталось попыток: $attempts\n", '31');
            }
        }
    }

    echo "\n" . color("🔐 Secret Diary (PHP)\n", '36');
    echo "1. Добавить запись\n";
    echo "2. Показать все записи\n";
    echo "3. Поиск записей\n";
    echo "4. Редактировать запись\n";
    echo "5. Удалить запись\n";
    echo "6. Экспорт в CSV\n";
    echo "7. Экспорт в JSON\n";
    echo "8. Сменить PIN-код\n";
    echo "9. Выход\n";
    echo "Выберите действие: ";
    $choice = trim(fgets(STDIN));

    switch ($choice) {
        case '1':
            echo "Категория (личное/работа/идеи/другое): ";
            $category = trim(fgets(STDIN));
            $category = strtolower($category);
            if (!in_array($category, ['личное', 'работа', 'идеи', 'другое'])) {
                $category = 'другое';
            }
            echo "Текст записи: ";
            $text = trim(fgets(STDIN));
            $id = count($diary['entries']) + 1;
            $date = date('Y-m-d H:i:s');
            $diary['entries'][] = ['id' => $id, 'date' => $date, 'category' => $category, 'text' => $text];
            saveDiary($diary);
            $lastActivity = time();
            echo color("✅ Запись добавлена (ID: $id)\n", '32');
            break;
        case '2':
            if (empty($diary['entries'])) {
                echo color("Нет записей.\n", '33');
            } else {
                printf(color("%-4s %-20s %-12s %-50s\n", '36'), "ID", "Дата", "Категория", "Текст");
                echo str_repeat("-", 90) . "\n";
                foreach ($diary['entries'] as $e) {
                    $text = strlen($e['text']) > 50 ? substr($e['text'], 0, 50) : $e['text'];
                    printf("%-4d %-20s %-12s %-50s\n", $e['id'], $e['date'], $e['category'], $text);
                }
            }
            break;
        case '3':
            echo "Ключевое слово для поиска: ";
            $keyword = trim(fgets(STDIN));
            $results = array_filter($diary['entries'], function($e) use ($keyword) {
                return stripos($e['text'], $keyword) !== false;
            });
            if (empty($results)) {
                echo color("Ничего не найдено.\n", '33');
            } else {
                foreach ($results as $e) {
                    echo "{$e['id']}: {$e['date']} | {$e['category']} | {$e['text']}\n";
                }
            }
            break;
        case '4':
            if (empty($diary['entries'])) {
                echo color("Нет записей.\n", '33');
                break;
            }
            foreach ($diary['entries'] as $e) {
                echo "{$e['id']}: {$e['date']} | {$e['category']} | {$e['text']}\n";
            }
            echo "Введите ID для редактирования: ";
            $id = (int) trim(fgets(STDIN));
            foreach ($diary['entries'] as &$e) {
                if ($e['id'] == $id) {
                    echo "Новый текст: ";
                    $e['text'] = trim(fgets(STDIN));
                    saveDiary($diary);
                    $lastActivity = time();
                    echo color("✅ Запись обновлена.\n", '32');
                    break 2;
                }
            }
            echo color("❌ Запись не найдена.\n", '31');
            break;
        case '5':
            if (empty($diary['entries'])) {
                echo color("Нет записей.\n", '33');
                break;
            }
            foreach ($diary['entries'] as $e) {
                echo "{$e['id']}: {$e['date']} | {$e['category']} | {$e['text']}\n";
            }
            echo "Введите ID для удаления: ";
            $id = (int) trim(fgets(STDIN));
            foreach ($diary['entries'] as $i => $e) {
                if ($e['id'] == $id) {
                    array_splice($diary['entries'], $i, 1);
                    saveDiary($diary);
                    $lastActivity = time();
                    echo color("✅ Запись удалена.\n", '32');
                    break 2;
                }
            }
            echo color("❌ Запись не найдена.\n", '31');
            break;
        case '6':
            if (empty($diary['entries'])) {
                echo color("Нет записей для экспорта.\n", '33');
                break;
            }
            $fp = fopen('diary_export.csv', 'w');
            fputcsv($fp, ['ID', 'Дата', 'Категория', 'Текст']);
            foreach ($diary['entries'] as $e) {
                fputcsv($fp, [$e['id'], $e['date'], $e['category'], $e['text']]);
            }
            fclose($fp);
            echo color("💾 Экспорт CSV: diary_export.csv\n", '32');
            break;
        case '7':
            if (empty($diary['entries'])) {
                echo color("Нет записей для экспорта.\n", '33');
                break;
            }
            file_put_contents('diary_export.json', json_encode($diary['entries'], JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE));
            echo color("💾 Экспорт JSON: diary_export.json\n", '32');
            break;
        case '8':
            echo "Текущий PIN: ";
            $old = trim(fgets(STDIN));
            echo "Новый PIN (4-6 цифр): ";
            $newPin = trim(fgets(STDIN));
            if (isDigit($newPin) && strlen($newPin) >= 4 && strlen($newPin) <= 6) {
                echo "Повторите новый PIN: ";
                $confirm = trim(fgets(STDIN));
                if ($newPin == $confirm) {
                    if (hashPin($old) == $diary['pin_hash']) {
                        $diary['pin_hash'] = hashPin($newPin);
                        saveDiary($diary);
                        echo color("✅ PIN изменён.\n", '32');
                    } else {
                        echo color("❌ Неверный текущий PIN.\n", '31');
                    }
                } else {
                    echo color("❌ PIN не совпадают.\n", '31');
                }
            } else {
                echo color("❌ PIN должен быть 4-6 цифр.\n", '31');
            }
            break;
        case '9':
            echo "До свидания!\n";
            exit(0);
        default:
            echo color("Неверный выбор.\n", '31');
    }
}
?>
