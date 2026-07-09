# yp-ffi — обработка изображений через плагины на FFI

Утилита командной строки, которая применяет к изображению эффект, реализованный
в виде динамической библиотеки. Основной бинарь (`image_processor`)
загружает нужный плагин через FFI (`libloading`) и передаёт ему сырые пиксели
и JSON с параметрами.

Доступные плагины:

- **mirror** — отражение по горизонтали и/или вертикали
- **blur** — гауссово размытие
- **grayscale** — перевод в градации серого

## Сборка

```bash
cargo build
```

Плагины собираются как `cdylib` и по умолчанию ищутся в `target/debug`
(там же, где их складывает `cargo build`). Для релизной сборки указывайте
`--plugin-path target/release`.

## Использование

```bash
cargo run -p image_processor -- \
  --input <файл> \
  --output <файл> \
  --plugin <mirror|blur|grayscale> \
  --params <файл-с-json>
```

Аргументы:

| Флаг             | Обязателен | По умолчанию   | Описание                                        |
| ---------------- | ---------- | -------------- | ----------------------------------------------- |
| `-i`, `--input`  | нет        | `-` (stdin)    | путь к исходному изображению                    |
| `-o`, `--output` | нет        | `-` (stdout)   | куда сохранить результат                        |
| `-p`, `--plugin` | да         | —              | какой плагин применить                          |
| `--params`       | нет        | `-` (stdin)    | путь к JSON-файлу с параметрами плагина         |
| `--plugin-path`  | нет        | `target/debug` | папка, где искать собранные библиотеки плагинов |

В репозитории есть тестовое изображение `img.png` и готовые примеры параметров
в папке `params/`.

## Примеры

### Зеркальное отражение

`params/mirror.json`:

```json
{
  "horizontal": true,
  "vertical": false
}
```

```bash
cargo run -p image_processor -- \
  --input img.png \
  --output out_mirror.png \
  --plugin mirror \
  --params params/mirror.json
```

### Размытие

Плагин `blur` принимает единственный параметр `sigma` — стандартное отклонение
гауссова ядра (чем больше значение, тем сильнее размытие). Если ключ не
указан, используется `0.0` (минимальное размытие согласно реализации крейта
`image`).

```json
{
  "sigma": 4.0
}
```

```bash
cargo run -p image_processor -- \
  --input img.png \
  --output out_blur.png \
  --plugin blur \
  --params params/blur.json
```

### Оттенки серого

У плагина `grayscale` нет настраиваемых параметров, но ему всё равно нужно
передать пустой JSON-объект:

```json
{}
```

```bash
cargo run -p image_processor -- \
  --input img.png \
  --output out_gray.png \
  --plugin grayscale \
  --params <(echo '{}')
```

### Работа через stdin/stdout

`--input`, `--output` и `--params` по умолчанию работают через `-`, то есть
поддерживают конвейеры:

```bash
cat img.png | cargo run -p image_processor -- \
  --plugin mirror \
  --params params/mirror.json > out.png
```

## Логирование

Уровень логов управляется переменной окружения `RUST_LOG` (стандартный
`tracing_subscriber::EnvFilter`), по умолчанию — `info`:

```bash
RUST_LOG=debug cargo run -p image_processor -- --input img.png --plugin blur --params params/blur.json --output out.png
```

## Структура репозитория

- `image_processor/` — CLI-бинарь, точка входа
- `plugin_interface/` — общий интерфейс: FFI-контракт (`process_image`),
  структуры параметров (`MirrorParams`, `BlurParams`, `GrayscaleParams`),
  ошибки, перечисление `Plugin`
- `mirror_plugin/`, `blur_plugin/`, `grayscale_plugin/` — сами плагины,
  каждый собирается в отдельную динамическую библиотеку
- `params/` — примеры JSON-параметров для плагинов
- `img.png` — тестовое изображение для проверки команд выше
