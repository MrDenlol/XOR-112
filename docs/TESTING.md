# Verification for 0.2.0

The release runs `cargo fmt --all -- --check`, `cargo test --release --locked` and `cargo build --release --locked`.

The suite contains 27 library tests and 19 UI tests. Coverage includes all eight majority input patterns, all 2,048 multiplexer data/address combinations, repeated template insertion and file round-trips, ALU arithmetic/editability, binary-only storage, grid snapping, undo/redo, theme/language persistence, and all combinations of hidden panels with restart checks in both languages.

Actual egui geometry is rendered for visual inspection. Local validation uses Windows x64 Rust GNU. GitHub Actions runs the full suite on Windows, Linux and macOS; the release job requires all three builds to pass. See [Desktop builds](https://github.com/MrDenlol/XOR-112/actions/workflows/build.yml) for the exact commit and results.

Automated UI tests inject mouse/keyboard events without a native desktop window. They do not constitute manual testing of every operating system or system file dialog. The release provides a Windows archive; Linux/macOS build outputs are workflow artifacts.
