use xor_112::{model::Circuit, storage};
fn main() {
    std::fs::create_dir_all("examples/circuits").unwrap();
    storage::save(
        std::path::Path::new("examples/circuits/alu32.binary"),
        &Circuit::alu32_example(),
    )
    .unwrap();
    storage::save(
        std::path::Path::new("examples/circuits/alu64.binary"),
        &Circuit::alu64_example(),
    )
    .unwrap();
    storage::save(
        std::path::Path::new("examples/circuits/half-adder.binary"),
        &Circuit::half_adder(),
    )
    .unwrap();
    storage::save(
        std::path::Path::new("examples/circuits/full-adder.binary"),
        &Circuit::full_adder(),
    )
    .unwrap();
}
