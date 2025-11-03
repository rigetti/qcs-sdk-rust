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
- replace features of `rigetti-pyo3` that were subsumed by modern PyO3
- switch to copy/pickle via `__getnewargs__`
- add linting scripts, auto-stub generation, and CI matrices for Python 3.10–3.13.

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

Common replacements:

* Class exposure → `#[cfg_attr(feature = "python", pyclass(module = "quil", name = "TypeName"))]`
    (and similar for enums)
* Constructor → `#[cfg(feature = "python")] #[pymethods] impl TypeName { #[new] fn new(…) -> Self { … } }`
    (or use the `pickleable_new` macro, which would need to be copied over, too)
* Methods → inside `#[pymethods]` blocks (split as needed)
* Properties → `#[getter] fn x(&self) -> …` and `#[setter] fn set_x(&mut self, v: …)`
    (but try for `get_all` or `#[get]` if possible)

---

## Provide a `#[pymodule]` that wires up the package surface

**Pattern:** create a single module entry point that registers classes and functions when `python` is enabled.

```rust
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_quil")]
fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use crate::qcs_sdk::errors;

    let py = m.py();

    m.add_wrapped(wrap_pymodule!(submodule::qcs_sdk::init_submodule))?;

    m.add_class::<Foo>()?;
    m.add_class::<Bar>()?;
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
* generated Python stub files (`python/quil/*.pyi`) 

---

## Update CI 

* Run checks and semver script (add a “Check quil API” job).
* Run tests on 3.10-3.13 across macOS/Linux/Windows.
* Add a stub validation step (e.g., mypy `stubtest`) once your stubs are in place

---

## Housekeeping

* Replace old references to the separate Python crate in docs/Makefiles with the integrated layout.
* Ensure any `Makefile.toml` tasks and scripts that build docs, run linting, or generate stubs
    point at the main crate paths, not the deleted Python crate.

