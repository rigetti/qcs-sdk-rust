use pyo3::{exceptions::PyRuntimeError, pymethods, Py, PyAny, PyResult, Python};
use pyo3_asyncio::tokio::future_into_py;
use qcs::qpu::Qcs;
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_type, wrap_error, ToPython, ToPythonError,
};

create_init_submodule! {
    classes: [PyQcsClient],
    errors: [QcsLoadError],
}

wrap_error! {
    LoadError(qcs::qpu::client::LoadError);
}
py_wrap_error!(client, LoadError, QcsLoadError, PyRuntimeError);

py_wrap_type! {
    PyQcsClient(Qcs) as "QcsClient";
}

#[pymethods]
impl PyQcsClient {
    // TODO: default arg
    #[new]
    pub fn new(py: Python<'_>, use_gateway: Option<bool>) -> PyResult<Self> {
        future_into_py(py, async move {
            let client = Qcs::load()
                .await
                .map_err(LoadError)
                .map_err(ToPythonError::to_py_err)?;

            let client = match use_gateway {
                None => client,
                Some(use_gateway) => client.with_use_gateway(use_gateway),
            };

            Python::with_gil(|py| <_ as ToPython<Py<PyAny>>>::to_python(&Self(client), py))
        })?
        .extract()
    }
}
