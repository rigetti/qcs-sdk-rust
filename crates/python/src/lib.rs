use ::qcs::configuration::Configuration;
use pythonize::pythonize;
use qcs::api;
use qcs_api::models::instruction_set_architecture::InstructionSetArchitecture;
use std::collections::HashMap;

use pyo3::{create_exception, exceptions::PyException, prelude::*};

create_exception!(qcs, InvalidConfigError, PyException);
create_exception!(qcs, ExecutionError, PyException);
create_exception!(qcs, TranslationError, PyException);
create_exception!(qcs, CompilationError, PyException);

// TODO Make these kw-only?
// TODO Return something more flexible than just a string (a Result object)
#[pyfunction]
fn compile(py: Python<'_>, quil: String, quantum_processor_isa: String) -> PyResult<&PyAny> {
    let quantum_processor_isa: InstructionSetArchitecture =
        serde_json::from_str(&quantum_processor_isa).unwrap();
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let config = Configuration::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let result = api::compile(&quil, quantum_processor_isa, &config)
            .await
            .map_err(|e| CompilationError::new_err(e.to_string()))?;
        Ok(Python::with_gil(|_py| result))
    })
}

#[pyfunction]
fn rewrite_arithmetic(py: Python<'_>, native_quil: String) -> PyResult<PyObject> {
    let result = api::rewrite_arithmetic(&native_quil).unwrap();
    let pyed = pythonize(py, &result).unwrap();
    Ok(pyed)
}

#[pyfunction]
fn build_patch_values(
    py: Python<'_>,
    recalculation_table: Vec<String>,
    memory: HashMap<String, Vec<f64>>,
) -> PyResult<PyObject> {
    let memory = memory
        .iter()
        .map(|(k, v)| (k.clone().into_boxed_str(), v.clone()))
        .collect();
    let patch_values = api::build_patch_values(recalculation_table, memory)
        .map_err(ExecutionError::new_err)
        .unwrap();
    let patch_values = pythonize(py, &patch_values).unwrap();
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
        // Is there a better way to map these patch_values keys? This
        // negates the whole purpose of [`submit`] using `Box<str>`,
        // instead of `String` directly, which normally would decrease
        // copies _and_ require less space, since str can't be extended.
        let patch_values = patch_values
            .iter()
            .map(|(k, v)| (k.clone().into_boxed_str(), v.clone()))
            .collect();
        let config = Configuration::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let job_id = api::submit(
            &program,
            patch_values,
            &quantum_processor_id,
            &config,
        )
        .await
        .map_err(|e| ExecutionError::new_err(e.to_string()))?;
        Ok(Python::with_gil(|_py| job_id))
    })
}

// TODO: Need to figure out how to pass the results back to Python
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
fn qcs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(rewrite_arithmetic, m)?)?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    m.add_function(wrap_pyfunction!(submit, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve_results, m)?)?;
    m.add_function(wrap_pyfunction!(build_patch_values, m)?)?;
    Ok(())
}
