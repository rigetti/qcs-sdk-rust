use pyo3::{exceptions::PyRuntimeError, pymethods, PyResult, Python};
use pyo3_asyncio::tokio::future_into_py;
use qcs::qpu::Qcs;
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_type, wrap_error, ToPython, ToPythonError,
};

create_init_submodule! {
    classes: [PyQcsClient],
    errors: [
        QcsGrpcClientError,
        QcsGrpcEndpointError,
        QcsGrpcError,
        QcsLoadError
    ],
}

wrap_error! {
    LoadError(qcs::qpu::client::LoadError);
}
py_wrap_error!(client, LoadError, QcsLoadError, PyRuntimeError);

wrap_error! {
    GrpcError(qcs::qpu::client::GrpcError);
}
py_wrap_error!(client, GrpcError, QcsGrpcError, PyRuntimeError);

wrap_error! {
    GrpcClientError(qcs::qpu::client::GrpcClientError);
}
py_wrap_error!(client, GrpcClientError, QcsGrpcClientError, PyRuntimeError);

wrap_error! {
    GrpcEndpointError(qcs::qpu::client::GrpcEndpointError);
}
py_wrap_error!(
    client,
    GrpcEndpointError,
    QcsGrpcEndpointError,
    PyRuntimeError
);

py_wrap_type! {
    PyQcsClient(Qcs) as "QcsClient";
}

#[pymethods]
impl PyQcsClient {
    #[new]
    #[args("/", use_gateway = "None")]
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

            Python::with_gil(|py| Self(client).to_python(py))
        })?
        .extract()
    }
}
