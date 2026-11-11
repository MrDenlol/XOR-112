use xor_112::{model::Circuit, storage};
fn main() {
    std::fs::create_dir_all("examples/circuits").unwrap();
    for (name, circuit) in [("majority", Circuit::majority()), ("mux8", Circuit::mux8())] {
        storage::save(
            &std::path::PathBuf::from(format!("examples/circuits/{name}.binary")),
            &circuit,
        )
        .unwrap();
    }
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
