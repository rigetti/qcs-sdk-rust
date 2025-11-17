# QCS SDK Crate Merge Plan

As with the `quil-rs` repository,
we wish to merge the (currently separate) Python bindings crate into the primary crate.
In `quil-rs`, the relevant commit is [this one](https://github.com/rigetti/quil-rs/commit/0050dae0dee053921911004d16b006e9581f8f89),
and more context is available in [this pull request](https://github.com/rigetti/quil-rs/commit/0050dae0dee053921911004d16b006e9581f8f89).
We need to do something similar in this repository (`qcs-sdk-rust`)
to merge its Python bindings crate into the main crate
while updating the PyO3 dependencies and integrating CI automation around API checks.

This document is a comprehensive plan for what specific changes need to happen,
using the details from the other merge work as a template.

## High-level Tasks

- eliminate the separate Python crate
- add PyO3 attributes directly in main crate, gated behind a `python` feature
- update to modern PyO3 patterns (removing `rigetti-pyo3` dependencies)
- switch to copy/pickle via `__getnewargs__`
- add linting scripts, auto-stub generation, and CI matrices for Python 3.10–3.13

---

# Step-by-Step Implementation Plan

## Preparation Steps

1. **Update main crate Cargo.toml** (`crates/lib/Cargo.toml`):
   - Add `python` feature flag
   - Add PyO3, pyo3-stub-gen dependencies (gated by `python` and `stubs` features)
   - Add `pyo3-build-config` to build-dependencies
   - Update crate-type to include `"cdylib"`
1. **Update root workspace configuration**:
    - Remove `crates/python` from `Cargo.toml` workspace members
    - Update any workspace-level scripts or tools
1. **Create Python module directory structure** in `crates/lib/src/`:
   - Create `crates/lib/python/` directory for output of stub generation
   - Move build-related files (e.g.`pyproject.toml`, `poetry.lock`) to `crates/lib/`
   - Add PyO3 build configuration when `python` feature is enabled
   - Copy any custom build logic from python crate's `build.rs`
   - Move `.flake8`, `.stubtest-allowlist` to main crate
   - Update any path references in config files
1. **Implement stub generation binary**:
    - Create `src/bin/stub_gen.rs` (similar to quil-rs)
    - Add stub generation logic for QCS SDK types
    - Create `python/qcs_sdk/` directory for stub files
1. **Add linting script**:
    - Copy and adapt linting script from quil-rs
    - Update for QCS SDK module structure
    - Add to `scripts/` directory
1. **Create Makefile.toml** (similar to quil-rs):
    - Add tasks: `install-deps`, `generate-stubs`, `package-qcs`, `install-qcs`
    - Add tasks: `pytest`, `stubtest`, `lint-qcs`, `test-qcs`, `test-all`
    - Add `check-api` task for API compatibility checking
1. **Set up basic Python module registration**:
   - Add `python.rs` file for the Python module root
   - Implement top-level `#[pymodule]` function similar to quil-rs pattern and register modules
   - Create `#[pymodule]` functions for each module
1. **Migrate core modules** from `crates/python` to `crates/lib/python`:
   - For wrappers in `crates/python/src/`,
   instead annotate their corresponding Rust structs with PyO3 attributes,
   gated appropriately, following the pattern in `quil-rs`
1. **Migrate Python tests**:
    - Move `crates/python/tests/` → `crates/lib/tests_py/`
    - Update import statements in test files
    - Ensure tests work with new module structure

## CI/CD Updates

1. **Update GitHub Actions workflows**:
    - Modify CI to test Python builds with different Python versions (3.10-3.13)
    - Add stub generation and validation steps
    - Add API compatibility checking
    - Update artifact publishing for combined crate
1. **Update documentation build**:
    - Modify doc generation to handle merged crate
    - Update paths in documentation workflows

## Final Steps

1. **Delete python crate directory**:
    - Remove `crates/python/` entirely
    - Verify no lingering references exist
1. **Update documentation**:
    - Update README.md files
    - Update CONTRIBUTING.md
    - Add migration notes for users
    - Update examples and usage instructions
1. **Version alignment**:
    - Update `knope.toml`
    - Ensure version numbers are coordinated
    - Update changelog with merge information

---

# Patterns to apply 

## Gate all Python-facing code behind the `python` cargo feature

**Pattern:** annotated exported code with PyO3 attributes, gated by `#[cfg(feature = "python")]`
and/or `#[cfg_attr(feature = "python", …)]`.

**Why:** The commit moved PyO3 attributes into the main crate and gated them with a `python` feature.

Examples and sub-tasks:

### Annotate code with PyO3 attributes 

```rust
// in a file exposing Python things...
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "package.submodule", eq, frozen, hash, get_all, subclass)
)]
pub struct Foo { /* … */ }
```

### Split Rust-only methods from shared methods

```rust
impl Foo { /* … */ }

#[cfg_attr(feature = "python", pyo3::pymethods)]
#[cfg_attr(not(feature = "python"), strip_pyo3)]
impl Foo { /* … */ }

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
#[cfg_attr(not(feature = "python"), strip_pyo3)]
impl Foo {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bar(&mut self) {
        /* ... */
    }
}
```

### Put Python-only methods in a Python-specific module 

Include that module in the crate graph, gated by the `python` feature.

```rust
// in an existing module...
#[cfg(feauture = "python")]
use mod qcs_sdk;

// in qcs_sdk.rs module file...
#[pymethods]
impl Foo { /* … */ }

```

---

## Replace `rigetti-pyo3` helpers with **native PyO3** attributes

**Pattern:** anywhere the old Python crate/macros/helpers were used,
re-express with first-party PyO3 attributes on the Rust types.

**Note:** The current QCS SDK Python crate already uses modern PyO3 patterns,
so this step involves migrating existing PyO3 code rather than replacing `rigetti-pyo3`.

Common patterns to apply:

* Class exposure → `#[cfg_attr(feature = "python", pyclass(module = "qcs_sdk", name = "TypeName"))]`
    (and similar for enums)
* Constructor → `#[cfg(feature = "python")] #[pymethods] impl TypeName { #[new] fn new(…) -> Self { … } }`
    (or use the `pickleable_new` macro, which would need to be copied over, too)
* Methods → inside `#[pymethods]` blocks (split as needed)
* Properties → `#[getter] fn x(&self) -> …` and `#[setter] fn set_x(&mut self, v: …)`
    (but try for `get_all` or `#[get]` if possible)
* Async methods → use `pyo3-asyncio` patterns for async functions

---

## Provide a `#[pymodule]` that wires up the package surface

**Pattern:** create a single module entry point that registers classes and functions when `python` is enabled.

```rust
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_qcs_sdk")]
fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use crate::python::errors;

    let py = m.py();

    // Add submodules
    m.add_wrapped(wrap_pymodule!(crate::python::client::init_submodule))?;
    m.add_wrapped(wrap_pymodule!(crate::python::compiler::init_submodule))?;
    m.add_wrapped(wrap_pymodule!(crate::python::qpu::init_submodule))?;
    m.add_wrapped(wrap_pymodule!(crate::python::qvm::init_submodule))?;

    // Add main classes
    m.add_class::<crate::Client>()?;
    m.add_class::<crate::Executable>()?;
    m.add_class::<crate::ExecutionData>()?;
    // ...

    m.add("QscSdkError", py.get_type::<errors::QcsSdkError>())?;
    //...
    
    let sys = PyModule::import(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.downcast_into()?;
    sys_modules.set_item("qcs_sdk.submodule", m.getattr("submodule")?)?;
    
    fix_complex_enums!(
        py,
        ComplexA,
        // ...
    );
    
    Ok(())
}
```

**Why:** This mirrors the integrated layout where the main crate builds the Python module when `python` is set.

---

## Migrate **copy/pickling** to `__getnewargs__`

**Pattern:** for Python `copy.copy`, `copy.deepcopy`, and pickle support,
implement `__getnewargs__` (and possibly `__getnewargs_ex__` if absolutely necessary).
Copy over and make use of `pickleable_new` macro whenever possible.

```rust
pickleable_new! {
    impl Foo {
        pub fn new(bar: int, baz: String);
    }
}
```

**Notes:**

* The tuple you return **must** match the `#[new]` constructor.
* Prefer reconstructing from immutable/primitive data you can round-trip.

It is necessary to write the `__getnewargs__` method explicitly for enums:

```rust
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl FooEnum {
    #[gen_stub(override_return_type(type_repr = "tuple[int | float | Bar]"))]
    fn __getnewargs__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        match self {
            Self::LiteralInteger(value) => (value,).into_pyobject(py),
            Self::LiteralReal(value) => (value,).into_pyobject(py),
            Self::Bar(value) => (value.clone(),).into_pyobject(py),
        }
    }
}
```

---

## Make **types/stubs** line up and be auto-generatable

**Pattern:** annotate all exported symbols; keep Python-visible API centralized and predictable.

* a linter to catch missing `#[pyclass]` / `#[pymethods]` coverage,
* generated Python stub files (`python/qcs_sdk/*.pyi`) 

---

## Update CI 

* Run checks and semver script (add a "Check QCS SDK API" job).
* Run tests on 3.10-3.13 across macOS/Linux/Windows.
* Add a stub validation step (e.g., mypy `stubtest`) once your stubs are in place

---

## Housekeeping

* Replace old references to the separate Python crate in docs/Makefiles with the integrated layout.
* Ensure any `Makefile.toml` tasks and scripts that build docs, run linting, or generate stubs
    point at the main crate paths, not the deleted Python crate.

