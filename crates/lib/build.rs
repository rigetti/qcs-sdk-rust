fn main() {
    #[cfg(feature = "python")]
    {
        pyo3_build_config::add_extension_module_link_args();
        pyo3_build_config::add_python_framework_link_args();
    }
    built::write_built_file().expect("Failed to acquire build-time information")
}
