/// Note, we put `pyo3-tracing-subscriber` stub build behind a feature flag, so that stubs are not
/// rewritten by default. When an actual build is needed, we use Cargo make commands so that we
/// can properly format the stub files after building them.
#[cfg(feature = "pyo3-tracing-subscriber-build")]
fn build_pyo3_tracing_subscriber_stubs() {
    use pyo3_tracing_subscriber::stubs::write_stub_files;
    use std::path::PathBuf;

    let tracing_subscriber_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("qcs_sdk/_tracing_subscriber");
    write_stub_files("qcs_sdk", "_tracing_subscriber", &tracing_subscriber_path)
        .expect("Failed to write pyo3-tracing-subscriber stub files");
}

fn main() {
    pyo3_build_config::add_extension_module_link_args();
    #[cfg(feature = "pyo3-tracing-subscriber-build")]
    build_pyo3_tracing_subscriber_stubs();
}
