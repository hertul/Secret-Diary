# secret_diary.rb — Ruby версия

require 'json'
require 'date'
require 'digest'

DATA_FILE = 'secret_diary.json'
LOCK_TIMEOUT = 300

class Entry
  attr_accessor :id, :date, :category, :text

  def initialize(id, date, category, text)
    @id = id
    @date = date
    @category = category
    @text = text
  end

  def to_h
    { id: @id, date: @date, category: @category, text: @text }
  end

  def self.from_h(h)
    new(h[:id], h[:date], h[:category], h[:text])
  end
end

class SecretDiary
  attr_reader :entries, :pin_hash, :locked

  def initialize
    @entries = []
    @pin_hash = nil
    @locked = true
    @last_activity = Time.now
    load
  end

  def load
    if File.exist?(DATA_FILE)
      begin
        data = JSON.parse(File.read(DATA_FILE), symbolize_names: true)
        @pin_hash = data[:pin_hash]
        @entries = (data[:entries] || []).map { |e| Entry.from_h(e) }
      rescue
        @pin_hash = nil
        @entries = []
      end
    end
  end

  def save
    data = { pin_hash: @pin_hash, entries: @entries.map(&:to_h) }
    File.write(DATA_FILE, JSON.pretty_generate(data))
  end

  def hash_pin(pin)
    Digest::SHA256.hexdigest(pin)
  end

  def check_pin(pin)
    if @pin_hash.nil?
      @pin_hash = hash_pin(pin)
      save
      return true
    end
    hash_pin(pin) == @pin_hash
  end

  def unlock(pin)
    if check_pin(pin)
      @locked = false
      @last_activity = Time.now
      true
    else
      false
    end
  end

  def change_pin(old_pin, new_pin)
    if check_pin(old_pin)
      @pin_hash = hash_pin(new_pin)
      save
      true
    else
      false
    end
  end

  def check_lock
    if Time.now - @last_activity > LOCK_TIMEOUT
      @locked = true
      true
    else
      false
    end
  end

  def add_entry(category, text)
    id = @entries.size + 1
    date = Time.now.strftime("%Y-%m-%d %H:%M:%S")
    @entries << Entry.new(id, date, category, text)
    save
    @last_activity = Time.now
    id
  end

  def list_all
    if @entries.empty?
      puts "\e[33mНет записей.\e[0m"
      return
    end
    printf "\e[36m%-4s %-20s %-12s %-50s\e[0m\n", "ID", "Дата", "Категория", "Текст"
    puts "-" * 90
    @entries.each do |e|
      text = e.text.length > 50 ? e.text[0...50] : e.text
      printf "%-4d %-20s %-12s %-50s\n", e.id, e.date, e.category, text
    end
  end

  def search(keyword)
    results = @entries.select { |e| e.text.downcase.include?(keyword.downcase) }
    if results.empty?
      puts "\e[33mНичего не найдено.\e[0m"
      return
    end
    results.each { |e| puts "#{e.id}: #{e.date} | #{e.category} | #{e.text}" }
  end

  def delete(id)
    found = @entries.find { |e| e.id == id }
    if found
      @entries.delete(found)
      save
      @last_activity = Time.now
      true
    else
      false
    end
  end

  def edit(id, new_text)
    entry = @entries.find { |e| e.id == id }
    if entry
      entry.text = new_text
      save
      @last_activity = Time.now
      true
    else
      false
    end
  end

  def export_csv(filename = 'diary_export.csv')
    if @entries.empty?
      puts "\e[33mНет записей для экспорта.\e[0m"
      return
    end
    File.open(filename, 'w') do |f|
      f.puts "ID,Дата,Категория,Текст"
      @entries.each do |e|
        f.puts "#{e.id},#{e.date},#{e.category},\"#{e.text}\""
      end
    end
    puts "\e[32m💾 Экспорт CSV: #{filename}\e[0m"
  end

  def export_json(filename = 'diary_export.json')
    if @entries.empty?
      puts "\e[33mНет записей для экспорта.\e[0m"
      return
    end
    File.write(filename, JSON.pretty_generate(@entries.map(&:to_h)))
    puts "\e[32m💾 Экспорт JSON: #{filename}\e[0m"
  end
end

def is_digit?(s)
  s.match?(/^\d+$/)
end

def main
  diary = SecretDiary.new
  puts "\e[36m🔐 Secret Diary (Ruby)\e[0m"

  if diary.pin_hash.nil?
    puts "\e[33mПервый запуск! Установите PIN-код (4-6 цифр):\e[0m"
    loop do
      print "PIN: "
      pin = gets.chomp
      if is_digit?(pin) && pin.length >= 4 && pin.length <= 6
        print "Повторите PIN: "
        confirm = gets.chomp
        if pin == confirm
          diary.pin_hash = diary.hash_pin(pin)
          diary.save
          puts "\e[32m✅ PIN установлен!\e[0m"
          diary.locked = false
          break
        else
          puts "\e[31m❌ PIN не совпадают.\e[0m"
        end
      else
        puts "\e[31m❌ PIN должен быть 4-6 цифр.\e[0m"
      end
    end
  end

  attempts = 3
  while diary.locked
    if attempts <= 0
      puts "\e[31m❌ Слишком много неудачных попыток. Выход.\e[0m"
      exit 1
    end
    print "Введите PIN-код: "
    pin = gets.chomp
    if diary.unlock(pin)
      puts "\e[32m✅ Добро пожаловать!\e[0m"
      break
    else
      attempts -= 1
      puts "\e[31m❌ Неверный PIN. Осталось попыток: #{attempts}\e[0m"
    end
  end

  loop do
    if diary.check_lock
      puts "\n\e[33m🔒 Автоматическая блокировка (бездействие > 5 мин)\e[0m"
      attempts = 3
      while diary.locked
        if attempts <= 0
          puts "\e[31m❌ Слишком много неудачных попыток. Выход.\e[0m"
          exit 1
        end
        print "Введите PIN-код для разблокировки: "
        pin = gets.chomp
        if diary.unlock(pin)
          puts "\e[32m✅ Разблокировано!\e[0m"
          break
        else
          attempts -= 1
          puts "\e[31m❌ Неверный PIN. Осталось попыток: #{attempts}\e[0m"
        end
      end
    end

    puts "\n\e[36m🔐 Secret Diary (Ruby)\e[0m"
    puts "1. Добавить запись"
    puts "2. Показать все записи"
    puts "3. Поиск записей"
    puts "4. Редактировать запись"
    puts "5. Удалить запись"
    puts "6. Экспорт в CSV"
    puts "7. Экспорт в JSON"
    puts "8. Сменить PIN-код"
    puts "9. Выход"
    print "Выберите действие: "
    choice = gets.chomp

    case choice
    when "1"
      print "Категория (личное/работа/идеи/другое): "
      category = gets.chomp.downcase
      category = "другое" unless ["личное", "работа", "идеи", "другое"].include?(category)
      print "Текст записи: "
      text = gets.chomp
      id = diary.add_entry(category, text)
      puts "\e[32m✅ Запись добавлена (ID: #{id})\e[0m"
    when "2"
      diary.list_all
    when "3"
      print "Ключевое слово для поиска: "
      keyword = gets.chomp
      diary.search(keyword)
    when "4"
      diary.list_all
      print "Введите ID для редактирования: "
      id = gets.chomp.to_i
      print "Новый текст: "
      text = gets.chomp
      if diary.edit(id, text)
        puts "\e[32m✅ Запись обновлена.\e[0m"
      else
        puts "\e[31m❌ Запись не найдена.\e[0m"
      end
    when "5"
      diary.list_all
      print "Введите ID для удаления: "
      id = gets.chomp.to_i
      if diary.delete(id)
        puts "\e[32m✅ Запись удалена.\e[0m"
      else
        puts "\e[31m❌ Запись не найдена.\e[0m"
      end
    when "6"
      print "Имя CSV файла (по умолч. diary_export.csv): "
      filename = gets.chomp
      filename = "diary_export.csv" if filename.empty?
      diary.export_csv(filename)
    when "7"
      print "Имя JSON файла (по умолч. diary_export.json): "
      filename = gets.chomp
      filename = "diary_export.json" if filename.empty?
      diary.export_json(filename)
    when "8"
      print "Текущий PIN: "
      old = gets.chomp
      print "Новый PIN (4-6 цифр): "
      new_pin = gets.chomp
      if is_digit?(new_pin) && new_pin.length >= 4 && new_pin.length <= 6
        print "Повторите новый PIN: "
        confirm = gets.chomp
        if new_pin == confirm
          if diary.change_pin(old, new_pin)
            puts "\e[32m✅ PIN изменён.\e[0m"
          else
            puts "\e[31m❌ Неверный текущий PIN.\e[0m"
          end
        else
          puts "\e[31m❌ PIN не совпадают.\e[0m"
        end
      else
        puts "\e[31m❌ PIN должен быть 4-6 цифр.\e[0m"
      end
    when "9"
      puts "До свидания!"
      break
    else
      puts "\e[31mНеверный выбор.\e[0m"
    end
  end
end

main if __FILE__ == $0
