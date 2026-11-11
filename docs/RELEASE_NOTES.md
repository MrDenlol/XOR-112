# XOR-112 0.2.0

## Русский

- Новый интерфейс: лента команд, скруглённые панели, один холст и масштабирование только в нижней строке.
- Скрытие свойств и библиотеки по отдельности или вместе; холст расширяется, выбор сохраняется после перезапуска.
- Новые развёрнутые примеры: большинство от трёх переменных и мультиплексор 8:1.
- Раздел «Готовые схемы» для вставки всех примеров, кроме АЛУ. Каждый вентиль и провод остаётся редактируемым; вставка поддерживает сетку, отмену и повтор.
- Все акценты и сигналы синей темы синие; обновлены иконки и исправлен чёрный фон между панелями.
- Обновлены README, руководство и снимки экрана; добавлен оригинальный badge XERJ Hub.

**Windows:** распакуйте `XOR-112-v0.2.0-windows-x64.zip` и запустите `xor-112.exe`. Rust не требуется. Все шесть примеров входят в архив. Для графики нужен OpenGL 3.3.

Формат `.binary`, русский/английский интерфейсы, три темы и полностью редактируемые АЛУ на 32/64 бита сохранены. Прежние схемы совместимы. Настройки переносятся автоматически; схемы сохраняются через Ctrl+S.

## English

Redesigned ribbon and rounded panels, bottom-only zoom controls, persistent sidebar visibility, editable majority and 8:1 multiplexer examples, reusable circuit templates, fully blue theme accents/signals, refined icons and a corrected workspace background. Updated documentation includes the original XERJ Hub contributor badge.

Extract the Windows x64 ZIP and run `xor-112.exe`. All six examples are included. Existing `.binary` files remain compatible. Russian/English, all three themes and editable 32/64-bit ALUs are preserved.

Windows, Linux and macOS builds/tests run in GitHub Actions before release publication. The release package is for Windows; Linux/macOS binaries are available as workflow artifacts, and source build instructions are in README. Native manual testing on every OS is not implied by CI success.
