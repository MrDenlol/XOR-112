# Contributing / Участие

Issues and pull requests in Russian or English are welcome.

Before a change, describe the problem and a small reproducible example. Include the OS, app version and relevant `.binary` file where possible. Remove personal information from attachments.

## Development

```sh
cargo fmt --all -- --check
cargo test --release --locked
cargo build --release --locked
```

Keep simulation logic independent of the UI. Changes to arithmetic, graph validation or storage should include meaningful regression tests. Add both Russian and English UI text. Preserve compatibility with existing `.binary` files or explicitly document a format change.

For UI changes, include screenshots and describe how keyboard and mouse behavior was checked. Do not commit generated binaries, personal preferences, API tokens or build folders. Generate ALU files with `cargo run --release --locked --example export_examples`.

Submit focused pull requests describing the user-visible outcome and checks performed. Contributions are provided under the project's MIT license. Be respectful and discuss technical differences constructively.
