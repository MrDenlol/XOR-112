# Testing

Run `cargo test --release --locked` from the repository root. Release mode keeps large gate-level ALU tests practical.

The suite covers gate truth tables, graph validation, binary parsing, file round-trips, expanded ALU arithmetic, editing connections, settings persistence/migration and headless mouse/keyboard UI scenarios. Arithmetic results are compared with native integer reference calculations, including boundary values and invalid operation codes.

UI checks exercise snapping, renaming, frame visibility, zoom input, help, language/theme changes and restoring settings. Preview screenshots are rendered from the actual egui meshes; they are not mockups.

Local release validation used Windows x64 with Rust GNU. Native file dialogs and a full manual desktop interaction pass are not covered by headless tests. Linux/macOS results are reported separately by GitHub Actions.

Release 0.1.0 passed all build and test jobs on Windows, Ubuntu and macOS in [workflow run 42479445459](https://github.com/MrDenlol/XOR-112/actions/runs/42479445459). The release job also successfully generated the example circuits, packaged the Windows build and published the archive and SHA-256 checksum.
