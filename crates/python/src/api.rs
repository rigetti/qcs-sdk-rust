use pyo3::{
    create_exception,
    exceptions::{PyRuntimeError, PyValueError},
    pyfunction,
    types::{PyComplex, PyFloat, PyInt, PyString},
    Py, PyResult,
};
use qcs::api::{ExecutionResult, ExecutionResults, Register};
use qcs_api_client_openapi::models::GetQuiltCalibrationsResponse;
use rigetti_pyo3::{
    create_init_submodule, impl_repr, py_wrap_data_struct, py_wrap_type, py_wrap_union_enum,
};

use crate::{py_sync::py_function_sync_async, qpu::client::PyQcsClient};

create_init_submodule! {
    classes: [
        PyExecutionResult,
        PyExecutionResults,
        PyRegister
    ],
    errors: [
        ExecutionError,
        DeviceISAError
    ],
    funcs: [
        py_retrieve_results,
        py_retrieve_results_async
    ],
}

py_wrap_data_struct! {
    PyQuiltCalibrations(GetQuiltCalibrationsResponse) as "QuiltCalibrations" {
        quilt: String => Py<PyString>,
        settings_timestamp: Option<String> => Option<Py<PyString>>
    }
}
impl_repr!(PyQuiltCalibrations);

py_wrap_type! {
    PyExecutionResults(ExecutionResults) as "ExecutionResults";
}

py_wrap_union_enum! {
    PyRegister(Register) as "Register" {
        f64: F64 => Vec<Py<PyFloat>>,
        i16: I16 => Vec<Py<PyInt>>,
        i32: I32 => Vec<Py<PyInt>>,
        i8: I8 => Vec<Py<PyInt>>,
        complex64: Complex64 => Vec<Py<PyComplex>>
    }
}

py_wrap_type! {
    PyExecutionResult(ExecutionResult) as "ExecutionResult";
}

create_exception!(qcs, ExecutionError, PyRuntimeError);
create_exception!(qcs, DeviceISAError, PyValueError);

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn retrieve_results(
        job_id: String,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyExecutionResults> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::api::retrieve_results(&job_id, &quantum_processor_id, &client)
            .await
            .map(PyExecutionResults::from)
            .map_err(|e| ExecutionError::new_err(e.to_string()))
    }
}
