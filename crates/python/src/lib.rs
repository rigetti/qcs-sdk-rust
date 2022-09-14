use ::qcs::configuration::Configuration;
use pythonize::pythonize;
use qcs::api;
use qcs::qpu::quilc::TargetDevice;
use std::collections::HashMap;

use pyo3::{create_exception, exceptions::PyRuntimeError, prelude::*};

create_exception!(qcs, InvalidConfigError, PyRuntimeError);
create_exception!(qcs, ExecutionError, PyRuntimeError);
create_exception!(qcs, TranslationError, PyRuntimeError);
create_exception!(qcs, CompilationError, PyRuntimeError);

#[pyfunction]
fn compile(py: Python<'_>, quil: String, target_device: String) -> PyResult<&PyAny> {
    let target_device: TargetDevice = serde_json::from_str(&target_device)
        .map_err(|e| CompilationError::new_err(e.to_string()))?;
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let config = Configuration::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let result = api::compile(&quil, target_device, &config)
            .map_err(|e| CompilationError::new_err(e.to_string()))?;
        Ok(Python::with_gil(|_py| result))
    })
}

#[pyfunction]
fn rewrite_arithmetic(py: Python<'_>, native_quil: String) -> PyResult<PyObject> {
    let result = api::rewrite_arithmetic(&native_quil).map_err(TranslationError::new_err)?;
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
        let config = Configuration::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let result = api::translate(&native_quil, num_shots, &quantum_processor_id, &config)
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
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let config = Configuration::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let job_id = api::submit(&program, patch_values, &quantum_processor_id, &config)
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
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let config = Configuration::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let results = api::retrieve_results(&job_id, &quantum_processor_id, &config)
            .await
            .map_err(|e| ExecutionError::new_err(e.to_string()))?;
        let results = Python::with_gil(|py| {
            pythonize(py, &results).map_err(|e| ExecutionError::new_err(e.to_string()))
        })?;
        Ok(results)
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
    Ok(())
}
