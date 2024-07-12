use std::path::PathBuf;

use pyo3_tracing_subscriber::stubs::write_stub_files;

fn main() {
    pyo3_build_config::add_extension_module_link_args();
    let tracing_subscriber_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("qcs_sdk/_tracing_subscriber");
    write_stub_files("qcs_sdk", "_tracing_subscriber", &tracing_subscriber_path)
        .expect("Failed to write pyo3-tracing-subscriber stub files");
}
