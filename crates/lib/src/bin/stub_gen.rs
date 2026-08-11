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
    use std::path::{Path, PathBuf};

    pub fn all() -> Result<(), Box<dyn std::error::Error>> {
        let mut stub = qcs::python::stub_info()?;
        rigetti_pyo3::stubs::sort(&mut stub);
        stub.generate()?;
        fix_external_module_imports(&stub.python_root.join("qcs_sdk"))?;
        tracing_subscriber(&stub.python_root)?;
        Ok(())
    }

    /// Rewrite `import a.b.c` as `from a.b import c` in generated stubs, where the stub refers
    /// to the module as `c`.
    ///
    /// `pyo3_stub_gen` writes the `from ... import ...` form only for modules under our own
    /// package, and a plain `import a.b.c` for every other module — but it refers to types in
    /// those modules by the last component alone, as in `program.Program`. A dotted `import`
    /// binds only the leftmost name, so those references are undefined. This affects any type
    /// we borrow from `quil` or `qcs_api_client_common`.
    ///
    /// Only imports whose fully-qualified path is never used are rewritten, which leaves
    /// correct cases such as `import collections.abc` (used as `collections.abc.Sequence`)
    /// alone.
    fn fix_external_module_imports(dir: &Path) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                fix_external_module_imports(&path)?;
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "pyi") {
                continue;
            }

            let source = std::fs::read_to_string(&path)?;
            let mut fixed = String::with_capacity(source.len());
            for line in source.lines() {
                match line.strip_prefix("import ").and_then(|m| m.rsplit_once('.')) {
                    Some((parent, leaf))
                        if !source.contains(&format!("{parent}.{leaf}."))
                            && source.contains(&format!("{leaf}.")) =>
                    {
                        fixed.push_str(&format!("from {parent} import {leaf}"));
                    }
                    _ => fixed.push_str(line),
                }
                fixed.push('\n');
            }

            if fixed != source {
                std::fs::write(&path, fixed)?;
            }
        }
        Ok(())
    }

    fn tracing_subscriber(
        python_root: &PathBuf,
    ) -> Result<(), pyo3_tracing_subscriber_build::Error> {
        let tracing_subscriber_path = python_root.join("qcs_sdk/_tracing_subscriber");
        pyo3_tracing_subscriber_build::write_stub_files(
            "qcs_sdk",
            "_tracing_subscriber",
            &tracing_subscriber_path,
        )
    }
}
