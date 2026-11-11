# Example circuits

All six examples are available from the file dropdown in the application:

- `half-adder.binary`: editable half adder.
- `full-adder.binary`: editable full adder.
- `majority.binary`: three-input majority, Tanenbaum fig. 3.3.
- `mux8.binary`: eight-input multiplexer, fig. 3.10; ABC selects D0–D7, A is the most significant bit.
- `alu32.binary` and `alu64.binary`: fully expanded editable ALUs.

The first four are also insertable from the Circuit templates library category. Insertion preserves the current document and adds independent editable gates and wires.

Regenerate all files from the project root:

```sh
cargo run --release --locked --example export_examples
```

Multi-input gates are expanded into two-input gates. All circuits use ordinary one-bit connections and the existing `.binary` format.
