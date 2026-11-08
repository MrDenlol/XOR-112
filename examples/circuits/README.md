# Example circuits

The half-adder and full-adder are small editable examples. All four examples are also built into XOR-112.

Generate the large 32-bit and 64-bit ALUs from the repository root:

```sh
cargo run --release --locked --example export_examples
```

This writes `half-adder.binary`, `full-adder.binary`, `alu32.binary` and `alu64.binary` here. Generated ALUs are included in the Windows release archive. They use ordinary gates and one-bit connections; no hidden arithmetic macro is needed.
