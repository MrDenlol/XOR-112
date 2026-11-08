# Architecture

The simulator is a library; the native egui/eframe editor is the binary. Graph evaluation does not depend on the renderer.

| Module | Responsibility |
|---|---|
| `model.rs` | Gate types, graph validation, topological evaluation, truth tables |
| `netlist.rs` | Fully expanded ALU generator and arithmetic regression tests |
| `storage.rs` | Versioned JSON, validation and atomic file replacement |
| `editor.rs`, `ribbon.rs` | Canvas interaction, history and commands |
| `netlist_view.rs` | Large-graph navigation, input buses and evaluation cache |
| `preferences.rs`, `settings_view.rs` | Persistent preferences and legacy migration |
| `appearance.rs`, `i18n.rs`, `translations.json` | Themes and Russian/English text |
| `wire_animation.rs` | Decorative animation of high signals |
| `alu64.rs`, `alu_view.rs` | Compatibility with older functional ALU files |

## Data and evaluation

All wires carry one bit. A disconnected or invalid result is diagnostic absence, not a third logic value. Connections are validated for source/destination types, pin bounds, single input drivers and cycles. Evaluation follows a topological order.

The generated ALUs use ripple-carry arithmetic, partial products, restoring division and paired-bit integer square root. The `netlist` field holds bus labels and navigation metadata; it never substitutes for gate evaluation.

`.binary` is UTF-8 JSON. Signal values serialize as `"0"` or `"1"`. Identifiers and coordinates are metadata. Formats 1, 2 and 3 are supported; format 3 is used by expanded ALUs. Loading rejects files over 128 MiB, more than 250,000 nodes or 600,000 wires.

The renderer culls invisible geometry and simplifies large graphs at low zoom. Large-graph undo history keeps six snapshots. Neither optimization changes circuit evaluation.

## Portability

The UI uses eframe/egui with OpenGL; rfd supplies native/portal file dialogs. Platform-specific settings paths are isolated in `preferences.rs`. CI builds and tests Windows, Linux and macOS; inspect its current results before claiming support for a specific environment.
