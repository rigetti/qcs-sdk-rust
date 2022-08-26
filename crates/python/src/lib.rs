use ::qcs::configuration::Configuration;
use qcs::api;
use std::collections::HashMap;

use pyo3::{create_exception, exceptions::PyException, prelude::*};

create_exception!(qcs, InvalidConfigError, PyException);
create_exception!(qcs, ExecutionError, PyException);
create_exception!(qcs, TranslationError, PyException);

#[pyfunction]
fn translate(
    py: Python<'_>,
    program: String,
    num_shots: u16,
    quantum_processor_id: String,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let config = Configuration::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let result = api::translate(&program, num_shots, &quantum_processor_id, &config)
            .await
            .map_err(|e| TranslationError::new_err(e.to_string()))?;
        // TODO: pass full results, not just program
        Ok(Python::with_gil(|_py| result.program))
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
        let job_id = api::submit(&program, patch_values, &quantum_processor_id, &config)
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
        println!("{:?}", results);
        Ok(Python::with_gil(|_py| {
            results.execution_duration_microseconds
        }))
    })
}

#[pymodule]
fn qcs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    m.add_function(wrap_pyfunction!(submit, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve_results, m)?)?;
    Ok(())
}
