<p align="center"><img src="docs/images/banner.svg" alt="XOR-112" width="100%"></p>

<p align="center">
  <a href="https://github.com/MrDenlol/XOR-112/releases/latest"><img src="https://img.shields.io/github/v/release/MrDenlol/XOR-112" alt="Release" height="20"></a>
  <a href="https://github.com/MrDenlol/XOR-112/actions/workflows/build.yml"><img src="https://github.com/MrDenlol/XOR-112/actions/workflows/build.yml/badge.svg" alt="Desktop builds" height="20"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-16834a" alt="MIT license" height="20"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange" alt="Rust 2021" height="20">
  <img src="https://img.shields.io/badge/interface-RU%20%2F%20EN-blue" alt="Russian and English" height="20">
  <a href="https://hub.xerj.org/contributors/MrDenlol"><img src="https://hub.xerj.org/contributors/MrDenlol/badge.svg" alt="XERJ Corpus Hub contributor" height="20"></a>
</p>

<p align="center"><a href="README.md">Русский</a> · <strong>English</strong> · <a href="https://github.com/MrDenlol/XOR-112/releases">Download</a> · <a href="https://github.com/MrDenlol/XOR-112/actions">Build status</a></p>

**XOR-112** is a native Rust desktop editor for binary combinational logic. Build circuits from gates, toggle their inputs and see outputs update immediately. Its **32-bit and 64-bit ALUs are fully expanded, editable gate circuits**: every gate and wire can be changed.

![XOR-112 in English with the blue theme](docs/images/home-blue-en.png)

## New in 0.2.0

- Redesigned ribbon, rounded panels and a single canvas, with zoom controls only in the bottom bar.
- **Display → Hide properties / Hide component library**: expand the canvas to either side or both. Visibility settings persist across restarts.
- Editable **three-input majority** and **8:1 multiplexer** examples.
- A third library category, **Circuit templates**, inserts independent editable copies of any example except ALUs. Select a template, then click the canvas; grid snapping, one-step undo/redo and Escape cancellation are supported.
- Blue signals, wires and application icon in the blue theme; refined icons and a corrected workspace background.

The **`.binary`** format and existing circuits remain compatible. See the [changelog](CHANGELOG.md).

![Canvas without sidebars](docs/images/hidden-both.png)

## Features

- AND, OR, NOT, NAND, NOR, XOR, XNOR and BUF gates, plus input switches and output indicators.
- Drag, rename, connect, duplicate and delete elements; undo and redo.
- Line grid, snapping, zoom, optional element frames and animated active wires.
- Connection validation and truth tables for circuits with up to eight inputs.
- Half-adder, full-adder, three-input majority, 8:1 multiplexer and editable 32-bit / 64-bit ALU examples.
- Save/load using the open `.binary` JSON format.
- Russian and English interface; green, blue and gray themes; persistent display settings.

All signal values and operands are binary. The app runs locally without a server.

## Windows download

Download [XOR-112 0.2.0 for Windows x64](https://github.com/MrDenlol/XOR-112/releases/download/v0.2.0/XOR-112-v0.2.0-windows-x64.zip), extract it and run **`xor-112.exe`**. Rust is not needed to run a packaged build. An OpenGL 3.3 graphics driver is required.

Toggle A and B in the initial half-adder. Open **Home → ⌄** for other examples, and **Settings** to change language/theme. Press **F1** for shortcuts. Save your work with **Ctrl+S**; circuits are not autosaved.

## Circuit templates

Scroll the library or collapse Logic gates to reveal Circuit templates. Choose an example, then click the canvas. All inserted nodes and wires remain independently editable; Ctrl+Z undoes the entire insertion and Esc cancels placement.

The majority output is 1 when at least two of A, B, C are 1. In the multiplexer, ABC selects D0–D7 with A the most significant address bit. Multi-input gates are expanded into two-input gates.

## Editable ALUs

| Width | Elements | One-bit wires |
|---|---:|---:|
| 32 bits | 18,777 | 37,263 |
| 64 bits | 73,617 | 146,671 |

Operation codes: `000` addition, `001` subtraction, `010` multiplication, `011` integer division, `100` integer square root (rounded down). Operands are unsigned. R holds the result; HI holds the upper product word; REM holds the remainder. Carry, borrow, overflow, divide-by-zero, invalid-code and validity flags are exposed. Unsupported codes and division by zero return zero data with invalid status.

The shared gate evaluator computes every operation. There is no hidden arithmetic block in these examples. Use section navigation or element-name search to inspect and edit the large circuit.

## Build from source

Use Rust stable and your platform's native build tools:

```sh
cargo test --release --locked
cargo build --release --locked
cargo run --release --locked
```

Windows: Visual Studio Build Tools (C++ and Windows SDK) with Rust MSVC, or MinGW-w64 with Rust GNU. Output: `target/release/xor-112.exe`.

Ubuntu/Debian dependencies:

```sh
sudo apt install build-essential pkg-config libx11-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev libxcursor-dev libxi-dev libxrandr-dev xdg-desktop-portal xdg-desktop-portal-gtk
```

macOS requires Xcode Command Line Tools. Linux/macOS output: `target/release/xor-112`. Windows was tested locally; [Actions](https://github.com/MrDenlol/XOR-112/actions) reports the actual results for all three platforms. Linux/macOS packaging is not provided as a signed installer.

Examples are built into the application. Generate standalone files with:

```sh
cargo run --release --locked --example export_examples
```

Generated files appear in `examples/circuits/` and are included in the release archive. You can also pass a `.binary` file as the first command-line argument.

## Scope and documentation

This is a combinational logic simulator: electrical characteristics, propagation delays, clocks, flip-flops and memory are outside this version's scope. Cycles are rejected. Large circuits use a simplified overview and six undo snapshots.

Settings are stored under `XOR-112/settings.json` in Windows AppData, Linux XDG config or macOS Application Support. Existing `BinaryLogic` preferences are copied on first launch if no XOR-112 settings exist. Circuits remain compatible with the previous supported file versions.

See the [Russian user guide](docs/USER_GUIDE.ru.md), [architecture](docs/ARCHITECTURE.md), [testing](docs/TESTING.md), [changelog](CHANGELOG.md) and [contribution guide](CONTRIBUTING.md).

## XERJ Hub

The project is included in [XERJ Corpus Hub — MrDenlol](https://hub.xerj.org/contributors/MrDenlol). The original Hub badge appears at the top of this page.

## License

[MIT](LICENSE). Dependency notices are in [THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt). No third-party textbook or reference photograph is distributed with this project.
