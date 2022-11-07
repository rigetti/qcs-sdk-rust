use pythonize::pythonize;
use qcs::api;
use qcs::qpu::client::Qcs;
use qcs::qpu::quilc::{CompilerOpts, TargetDevice, DEFAULT_COMPILER_TIMEOUT};
use std::collections::HashMap;

use pyo3::{
    create_exception,
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PyDict,
};

create_exception!(qcs, InvalidConfigError, PyRuntimeError);
create_exception!(qcs, ExecutionError, PyRuntimeError);
create_exception!(qcs, TranslationError, PyRuntimeError);
create_exception!(qcs, CompilationError, PyRuntimeError);
create_exception!(qcs, RewriteArithmeticError, PyRuntimeError);
create_exception!(qcs, DeviceIsaError, PyValueError);

#[pyfunction(kwds = "**")]
fn compile<'a>(
    py: Python<'a>,
    quil: String,
    target_device: String,
    kwds: Option<&PyDict>,
) -> PyResult<&'a PyAny> {
    let target_device: TargetDevice =
        serde_json::from_str(&target_device).map_err(|e| DeviceIsaError::new_err(e.to_string()))?;

    let mut compiler_timeout = Some(DEFAULT_COMPILER_TIMEOUT);
    if let Some(kwargs) = kwds {
        if let Some(timeout_arg) = kwargs.get_item("timeout") {
            let timeout: Result<Option<u8>, _> = timeout_arg.extract();
            if let Ok(option) = timeout {
                compiler_timeout = option
            }
        }
    }

    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let options = CompilerOpts::default().with_timeout(compiler_timeout);
        let result = api::compile(&quil, target_device, &client, options)
            .map_err(|e| CompilationError::new_err(e.to_string()))?;
        Ok(Python::with_gil(|_py| result))
    })
}

#[pyfunction]
fn rewrite_arithmetic(py: Python<'_>, native_quil: String) -> PyResult<PyObject> {
    let native_program = native_quil
        .parse::<quil_rs::Program>()
        .map_err(|e| TranslationError::new_err(e.to_string()))?;
    let result = api::rewrite_arithmetic(native_program)
        .map_err(|e| RewriteArithmeticError::new_err(e.to_string()))?;
    let pyed = pythonize(py, &result).map_err(|e| TranslationError::new_err(e.to_string()))?;
    Ok(pyed)
}

#[pyfunction]
fn build_patch_values(
    py: Python<'_>,
    recalculation_table: Vec<String>,
    memory: HashMap<String, Vec<f64>>,
) -> PyResult<PyObject> {
    let memory = memory
        .into_iter()
        .map(|(k, v)| (k.into_boxed_str(), v))
        .collect();
    let patch_values = api::build_patch_values(&recalculation_table, &memory)
        .map_err(TranslationError::new_err)?;
    let patch_values =
        pythonize(py, &patch_values).map_err(|e| TranslationError::new_err(e.to_string()))?;
    Ok(patch_values)
}

#[pyfunction]
fn translate(
    py: Python<'_>,
    native_quil: String,
    num_shots: u16,
    quantum_processor_id: String,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let result = api::translate(&native_quil, num_shots, &quantum_processor_id, &client)
            .await
            .map_err(|e| TranslationError::new_err(e.to_string()))?;
        let result = Python::with_gil(|py| {
            pythonize(py, &result).map_err(|e| TranslationError::new_err(e.to_string()))
        })?;
        Ok(result)
    })
}

#[pyfunction]
fn submit(
    py: Python<'_>,
    program: String,
    patch_values: HashMap<String, Vec<f64>>,
    quantum_processor_id: String,
    use_gateway: bool,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?
            .with_use_gateway(use_gateway);
        let job_id = api::submit(&program, patch_values, &quantum_processor_id, &client)
            .await
            .map_err(|e| ExecutionError::new_err(e.to_string()))?;
        Ok(Python::with_gil(|_py| job_id))
    })
}

#[pyfunction]
fn retrieve_results(
    py: Python<'_>,
    job_id: String,
    quantum_processor_id: String,
    use_gateway: bool,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?
            .with_use_gateway(use_gateway);
        let results = api::retrieve_results(&job_id, &quantum_processor_id, &client)
            .await
            .map_err(|e| ExecutionError::new_err(e.to_string()))?;
        let results = Python::with_gil(|py| {
            pythonize(py, &results).map_err(|e| ExecutionError::new_err(e.to_string()))
        })?;
        Ok(results)
    })
}

#[pyfunction]
fn get_quilc_version(py: Python<'_>) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let version = api::get_quilc_version(&client)
            .map_err(|e| CompilationError::new_err(e.to_string()))?;
        Ok(version)
    })
}

#[pymodule]
fn qcs_sdk(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(rewrite_arithmetic, m)?)?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    m.add_function(wrap_pyfunction!(submit, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve_results, m)?)?;
    m.add_function(wrap_pyfunction!(build_patch_values, m)?)?;
    m.add_function(wrap_pyfunction!(get_quilc_version, m)?)?;
    Ok(())
}
