use std::{collections::HashMap, time::Duration};

use pyo3::{
    create_exception,
    exceptions::{PyRuntimeError, PyValueError},
    pyfunction,
    types::{PyComplex, PyDict, PyFloat, PyInt, PyString},
    Py, PyResult,
};
use qcs::api::{self, ExecutionResult, ExecutionResults, Register, TranslationResult};
use qcs_api_client_openapi::models::GetQuiltCalibrationsResponse;
use rigetti_pyo3::{
    create_init_submodule, impl_repr, py_wrap_data_struct, py_wrap_error, py_wrap_type,
    py_wrap_union_enum, wrap_error, ToPythonError,
};

use crate::{py_sync::py_function_sync_async, qpu::client::PyQcsClient};

create_init_submodule! {
    classes: [
        PyExecutionResult,
        PyExecutionResults,
        PyRegister,
        PyTranslationResult,
        PyQuiltCalibrations
    ],
    errors: [
        ExecutionError,
        TranslationError,
        CompilationError,
        RewriteArithmeticError,
        DeviceISAError,
        QCSListQuantumProcessorsError,
        QCSSubmitError,
        QCSGetQuiltCalibrationsError
    ],
    funcs: [
        py_translate,
        py_translate_async,
        py_submit,
        py_submit_async,
        py_retrieve_results,
        py_retrieve_results_async,
        py_list_quantum_processors,
        py_list_quantum_processors_async,
        py_get_quilt_calibrations,
        py_get_quilt_calibrations_async
    ],
}

py_wrap_data_struct! {
    PyTranslationResult(TranslationResult) as "TranslationResult" {
        program: String => Py<PyString>,
        ro_sources: Option<HashMap<String, String>> => Option<Py<PyDict>>
    }
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
create_exception!(qcs, TranslationError, PyRuntimeError);
create_exception!(qcs, CompilationError, PyRuntimeError);
create_exception!(qcs, RewriteArithmeticError, PyRuntimeError);
create_exception!(qcs, DeviceISAError, PyValueError);

wrap_error!(SubmitError(qcs::api::SubmitError));
py_wrap_error!(api, SubmitError, QCSSubmitError, PyRuntimeError);

wrap_error!(ListQuantumProcessorsError(
    qcs::api::ListQuantumProcessorsError
));
py_wrap_error!(
    api,
    ListQuantumProcessorsError,
    QCSListQuantumProcessorsError,
    PyRuntimeError
);

wrap_error!(GetQuiltCalibrationsError(
    qcs::api::GetQuiltCalibrationsError
));
py_wrap_error!(
    api,
    GetQuiltCalibrationsError,
    QCSGetQuiltCalibrationsError,
    PyRuntimeError
);

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn translate(
        native_quil: String,
        num_shots: u16,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
    ) -> PyResult<PyTranslationResult> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::api::translate(&native_quil, num_shots, &quantum_processor_id, &client)
            .await
            .map(PyTranslationResult::from)
            .map_err(|e| TranslationError::new_err(e.to_string()))
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    async fn submit(
        program: String,
        patch_values: HashMap<String, Vec<f64>>,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
    ) -> PyResult<String> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::api::submit(&program, patch_values, &quantum_processor_id, &client)
            .await
            .map_err(|e| ExecutionError::new_err(e.to_string()))
    }
}

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

py_function_sync_async! {
    #[pyfunction(client = "None", timeout = "None")]
    async fn list_quantum_processors(
        client: Option<PyQcsClient>,
        timeout: Option<f64>,
    ) -> PyResult<Vec<String>> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let timeout = timeout.map(Duration::from_secs_f64);
        api::list_quantum_processors(&client, timeout)
            .await
            .map_err(ListQuantumProcessorsError::from)
            .map_err(ListQuantumProcessorsError::to_py_err)
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None", timeout = "None")]
    async fn get_quilt_calibrations(
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
        timeout: Option<f64>,
    ) -> PyResult<PyQuiltCalibrations> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let timeout = timeout.map(Duration::from_secs_f64);
        api::get_quilt_calibrations(&quantum_processor_id, &client, timeout)
            .await
            .map(PyQuiltCalibrations::from)
            .map_err(GetQuiltCalibrationsError::from)
            .map_err(GetQuiltCalibrationsError::to_py_err)
    }
}
