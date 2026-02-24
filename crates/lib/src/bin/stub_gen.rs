//! This binary is used to generate Python stub files (type hints) for the `qcs_sdk` package.
//! For more information on why this exists as a separate binary rather than a build script,
//! see the [`pyo3-stub-gen`][] documentation.
//!
//! [`pyo3-stub-gen`]: https://github.com/Jij-Inc/pyo3-stub-gen

#[cfg(not(feature = "stubs"))]
fn main() {
    eprintln!("Executing this binary only makes sense with the --stubs feature enabled.");
}

#[cfg(feature = "stubs")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate::all()
}

#[cfg(feature = "stubs")]
mod generate {
    use std::path::PathBuf;

    pub fn all() -> Result<(), Box<dyn std::error::Error>> {
        let mut stub = qcs::python::stub_info()?;
        rigetti_pyo3::stubs::sort(&mut stub);
        stub.generate()?;
        tracing_subscriber(&stub.python_root)?;
        Ok(())
    }

    fn tracing_subscriber(
        python_root: &PathBuf,
    ) -> Result<(), pyo3_tracing_subscriber::stubs::Error> {
        let tracing_subscriber_path = python_root.join("qcs_sdk/_tracing_subscriber");
        pyo3_tracing_subscriber::stubs::write_stub_files(
            "qcs_sdk",
            "_tracing_subscriber",
            &tracing_subscriber_path,
        )
    }
}
