# XOR-112 0.1.0

## Русский

Первый публичный выпуск редактора бинарных логических схем на Rust.

- Восемь базовых вентилей; создание, редактирование и сохранение схем.
- Полностью редактируемые АЛУ на 32 и 64 бита с операциями +, −, ×, ÷ и целым квадратным корнем.
- Русский/английский интерфейс, три темы, сетка, привязка и анимация проводов.
- Сохранение настроек и перенос параметров из прежней версии BinaryLogic.

**Windows:** распакуйте ZIP и запустите `xor-112.exe`. Установка Rust не нужна. Все четыре примера находятся в `examples/circuits/` и доступны из меню программы. Требуется OpenGL 3.3.

## English

First public release of the Rust binary circuit editor. Includes eight basic gates, editable 32/64-bit gate-level ALUs, save/load, truth tables, Russian/English UI, three themes and persistent preferences.

Extract the Windows x64 ZIP and run `xor-112.exe`. No Rust installation is required. Examples are included and available from the app menu. OpenGL 3.3 is required.

This is a combinational simulator. Arithmetic is unsigned and integer-only. Circuits must be saved manually. Windows binaries are not code-signed. `SHA256SUMS.txt` contains archive integrity hashes.
