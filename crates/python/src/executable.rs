use pyo3::pymethods;
use qcs::Executable;
use rigetti_pyo3::py_wrap_type;

// Because Python is garbage-collected, no lifetimes can be guaranteed except `'static`.
py_wrap_type! {
    PyExecutable(Executable<'static, 'static>) as "Executable";
}

#[pymethods]
impl PyExecutable {}
