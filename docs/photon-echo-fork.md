# Изменения форка Photon Echo

База: distortionary-labs/8bf-host, коммит
`2a52154f160bb25c9ab2a77c29c7ddaccf0b8258`, версия пакета 0.1.0.
Лицензии MIT OR Apache-2.0 и исходная история сохранены.
Это форк открытого хоста; проприетарный код Dehancer в него не включён.

## Назначение изменений

Изменения разработаны в проекте
[photon-echo-tools](https://github.com/conga07/photon-echo-tools)
для Dehancer Photoshop Plugin 2.8.0 на macOS ARM64:

- Нативный macOS PiPL имеет заголовок version/count длиной 8 байт,
  без двухбайтной Windows-сигнатуры. Сканер теперь принимает оба представления.
- Scripting TEXT-дескрипторы Dehancer используют Carbon Handles. Добавлена их
  обработка через CoreServices; проверяются читаемость master pointer,
  принадлежность памяти allocator zone и обратный `RecoverHandle`.
- В новом экземпляре вызывается Parameters, когда private data ещё не создана,
  даже при наличии сохранённого блока параметров. Это устраняет наблюдавшееся
  падение Dehancer в ReadScriptParameters на нулевом gData.
- `macapp::ensure_application(activate)` не вызывает activate для silent jobs
  и использует Accessory activation policy. Интерактивные вызовы и About сохраняют
  обычную активацию. Сигнатура этого API изменилась: теперь нужен bool.
- `protocol::run_job_with_host(path, progress, &mut host)` позволяет сохранить
  FilterHost, модуль плагина и private data между заданиями. Существующий
  `run_job` создаёт новый FilterHost и сохраняет прежний одиночный контракт.

Сохранён диагностический opt-in `EIGHTBF_RESEARCH_WARMUP_MS`: до Prepare
обслуживает Cocoa-события, максимум 10 секунд. Он не включён по умолчанию;
ожидание 5 секунд в опыте не устранило водяной знак холодного silent-процесса.

## Использование постоянного хоста

```sh
cargo +1.97.1 build --example persistent_host
cargo +1.97.1 run --example persistent_host -- /jobs/first/request.json /jobs/second/request.json
```

Пример использует внутренний worker-протокол 8bf-host: заранее подготовленные
`request.json`, RGB-буферы и индивидуальные descriptors. Каждое задание пишет
ответ и результат в свой каталог. Это не CLI для TIFF или YAML `.dh`.
Все вызовы остаются на главном потоке; модуль между ними не выгружается.
Первый job может иметь `show_ui:true`, последующие — `false`.
Автоматическое нажатие OK, конвертация `.dh`, EXIF/ICC и публикация TIFF относятся
к утилите dh8bf в photon-echo-tools и сюда не переносились.

## Что подтверждено

На macOS 27.0.1 / Apple M1 после одного интерактивного вызова Dehancer с OK
два последующих silent-вызова в одном процессе с различными нативными состояниями
дали чистые RGB8-результаты, полностью совпавшие по пикселям с Affinity.
При каждом новом холодном процессе silent-результат имел водяной знак, хотя
интерактивный вызов показывал Activated. Внутренняя причина не установлена;
лицензирование и бинарник плагина не изменялись.

Это не обещание безоконного холодного старта. Проверена короткая последовательность,
не длительный batch и не все Photoshop-плагины. Установленный Dehancer PiPL
не объявляет RGB16; хост его отклонил. Эти сведения не ограничивают общую
поддержку 16 бит другими плагинами в upstream.
Silent playback может не возвращать recording descriptor (`null`); входной
дескриптор и состояние для записи — разные вещи.

## Проверки и сборка

```sh
cargo +1.97.1 fmt --check
cargo +1.97.1 test --lib host::
cargo +1.97.1 test --lib protocol::
cargo +1.97.1 build --example persistent_host
```

Нужны Rust/Cargo и инструменты сборки macOS. Cocoa/Carbon зависят от SDK macOS.
Новые тесты проверяют нативный PiPL и цикл чтения/изменения Carbon Handle,
включая некорректный адрес. Фотографии, пользовательские настройки и activation
данные не нужны для unit-тестов и в этот репозиторий не включены.

При переносе форка 16 ноября 2026 прошли 150 тестов `host::` и 8 `protocol::`,
`cargo fmt --check` и сборка примера `persistent_host`. Проверки проводились на
macOS ARM64; Windows/Linux после этих изменений отдельно не запускались.
