use std::{collections::HashMap, time::Duration};

use pyo3::{
    create_exception,
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    pyfunction,
    types::{PyComplex, PyDict, PyFloat, PyInt, PyList, PyString},
    Py, PyResult,
};
use qcs::{
    api::{
        get_quilt_calibrations, list_quantum_processors, ExecutionResult, ExecutionResults,
        Register, RewriteArithmeticResult, TranslationResult,
    },
    qpu::quilc::{CompilerOpts, TargetDevice, DEFAULT_COMPILER_TIMEOUT},
};
use qcs_api_client_openapi::models::GetQuiltCalibrationsResponse;
use rigetti_pyo3::{
    create_init_submodule, impl_repr, py_wrap_data_struct, py_wrap_error, py_wrap_type,
    py_wrap_union_enum, wrap_error, ToPython, ToPythonError,
};

use crate::qpu::client::PyQcsClient;

create_init_submodule! {
    classes: [
        PyExecutionResult,
        PyExecutionResults,
        PyRegister,
        PyRewriteArithmeticResult,
        PyTranslationResult
    ],
    errors: [
        ExecutionError,
        TranslationError,
        CompilationError,
        RewriteArithmeticError,
        DeviceIsaError,
        QcsListQuantumProcessorsError,
        QcsSubmitError,
        QcsGetQuiltCalibrationsError
    ],
    funcs: [
        compile,
        rewrite_arithmetic,
        translate,
        submit,
        retrieve_results,
        build_patch_values,
        get_quilc_version,
        py_list_quantum_processors,
        py_get_quilt_calibrations
    ],
}

py_wrap_data_struct! {
    PyRewriteArithmeticResult(RewriteArithmeticResult) as "RewriteArithmeticResult" {
        program: String => Py<PyString>,
        recalculation_table: Vec<String> => Py<PyList>
    }
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
create_exception!(qcs, DeviceIsaError, PyValueError);

wrap_error!(SubmitError(qcs::api::SubmitError));
py_wrap_error!(api, SubmitError, QcsSubmitError, PyRuntimeError);

wrap_error!(ListQuantumProcessorsError(
    qcs::api::ListQuantumProcessorsError
));
py_wrap_error!(
    api,
    ListQuantumProcessorsError,
    QcsListQuantumProcessorsError,
    PyRuntimeError
);

wrap_error!(GetQuiltCalibrationsError(
    qcs::api::GetQuiltCalibrationsError
));
py_wrap_error!(
    api,
    GetQuiltCalibrationsError,
    QcsGetQuiltCalibrationsError,
    PyRuntimeError
);

/// Get the keyword `key` value from `kwds` if it is of type `Option<T>` and it is present, else `None`.
/// Returns an error if a value is present but cannot be extracted into `T`.
fn get_kwd<'a, T: FromPyObject<'a>>(kwds: Option<&'a PyDict>, key: &str) -> PyResult<Option<T>> {
    kwds.and_then(|kwds| kwds.get_item(key))
        .map_or(Ok(None), PyAny::extract::<Option<T>>)
}

#[pyfunction(client = "None", kwds = "**")]
pub fn compile(
    quil: String,
    target_device: String,
    client: Option<PyQcsClient>,
    kwds: Option<&PyDict>,
) -> PyResult<String> {
    let target_device: TargetDevice =
        serde_json::from_str(&target_device).map_err(|e| DeviceIsaError::new_err(e.to_string()))?;

    let compiler_timeout = get_kwd(kwds, "timeout")?.or(Some(DEFAULT_COMPILER_TIMEOUT));
    let protoquil: Option<bool> = get_kwd(kwds, "protoquil")?;

    crate::utils::py_sync!(async move {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let options = CompilerOpts::default()
            .with_timeout(compiler_timeout)
            .with_protoquil(protoquil);
        qcs::api::compile(&quil, target_device, &client, options)
            .map_err(|e| CompilationError::new_err(e.to_string()))
    })
}

#[pyfunction]
pub fn rewrite_arithmetic(native_quil: String) -> PyResult<PyRewriteArithmeticResult> {
    let native_program = native_quil
        .parse::<quil_rs::Program>()
        .map_err(|e| TranslationError::new_err(e.to_string()))?;
    let result = qcs::api::rewrite_arithmetic(native_program)
        .map_err(|e| RewriteArithmeticError::new_err(e.to_string()))?;
    let pyed = result.into();
    Ok(pyed)
}

#[pyfunction]
pub fn build_patch_values(
    py: Python<'_>,
    recalculation_table: Vec<String>,
    memory: HashMap<String, Vec<f64>>,
) -> PyResult<Py<PyDict>> {
    let memory = memory
        .into_iter()
        .map(|(k, v)| (k.into_boxed_str(), v))
        .collect();
    let patch_values = qcs::api::build_patch_values(&recalculation_table, &memory)
        .map_err(TranslationError::new_err)?;
    patch_values
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect::<HashMap<_, _>>()
        .to_python(py)
}

#[pyfunction(client = "None")]
pub fn translate(
    native_quil: String,
    num_shots: u16,
    quantum_processor_id: String,
    client: Option<PyQcsClient>,
) -> PyResult<PyTranslationResult> {
    crate::utils::py_sync!(async move {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::api::translate(&native_quil, num_shots, &quantum_processor_id, &client)
            .await
            .map(PyTranslationResult::from)
            .map_err(|e| TranslationError::new_err(e.to_string()))
    })
}

#[pyfunction(client = "None")]
pub fn submit(
    program: String,
    patch_values: HashMap<String, Vec<f64>>,
    quantum_processor_id: String,
    client: Option<PyQcsClient>,
) -> PyResult<String> {
    crate::utils::py_sync!(async move {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::api::submit(&program, patch_values, &quantum_processor_id, &client)
            .await
            .map_err(|e| ExecutionError::new_err(e.to_string()))
    })
}

#[pyfunction(client = "None")]
pub fn retrieve_results(
    job_id: String,
    quantum_processor_id: String,
    client: Option<PyQcsClient>,
) -> PyResult<PyExecutionResults> {
    crate::utils::py_sync!(async move {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::api::retrieve_results(&job_id, &quantum_processor_id, &client)
            .await
            .map(PyExecutionResults::from)
            .map_err(|e| ExecutionError::new_err(e.to_string()))
    })
}

#[pyfunction(client = "None")]
pub fn get_quilc_version(client: Option<PyQcsClient>) -> PyResult<String> {
    crate::utils::py_sync!(async move {
        let client = PyQcsClient::get_or_create_client(client).await?;
        qcs::api::get_quilc_version(&client).map_err(|e| CompilationError::new_err(e.to_string()))
    })
}

#[pyfunction(client = "None", timeout = "None")]
#[pyo3(name = "list_quantum_processors")]
pub fn py_list_quantum_processors(
    client: Option<PyQcsClient>,
    timeout: Option<f64>,
) -> PyResult<Vec<String>> {
    crate::utils::py_sync!(async move {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let timeout = timeout.map(Duration::from_secs_f64);
        list_quantum_processors(&client, timeout)
            .await
            .map_err(ListQuantumProcessorsError::from)
            .map_err(ListQuantumProcessorsError::to_py_err)
    })
}

#[pyfunction(client = "None", timeout = "None")]
#[pyo3(name = "get_quilt_calibrations")]
pub fn py_get_quilt_calibrations(
    quantum_processor_id: String,
    client: Option<PyQcsClient>,
    timeout: Option<f64>,
) -> PyResult<PyQuiltCalibrations> {
    crate::utils::py_sync!(async move {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let timeout = timeout.map(Duration::from_secs_f64);
        get_quilt_calibrations(&quantum_processor_id, &client, timeout)
            .await
            .map(PyQuiltCalibrations::from)
            .map_err(GetQuiltCalibrationsError::from)
            .map_err(GetQuiltCalibrationsError::to_py_err)
    })
}
