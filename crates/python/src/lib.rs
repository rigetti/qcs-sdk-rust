use ::qcs::configuration::Configuration;
use qcs::api;
use std::collections::HashMap;

use pyo3::prelude::*;

#[pyfunction]
fn submit(
    py: Python<'_>,
    program: String,
    _patch_values: HashMap<String, Vec<f64>>,
    quantum_processor_id: String,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let values = HashMap::new();
        let config = Configuration::load().await.unwrap();
        let job_id = api::submit(&program, values, &quantum_processor_id, &config)
            .await
            .unwrap();
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
        let config = Configuration::load().await.unwrap();
        let results = api::retrieve_results(&job_id, &quantum_processor_id, &config)
            .await
            .unwrap();
        println!("{:?}", results);
        Ok(Python::with_gil(|_py| {
            results.execution_duration_microseconds
        }))
    })
}

#[pymodule]
fn qcs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(submit, m)?)?;
    m.add_function(wrap_pyfunction!(retrieve_results, m)?)?;
    Ok(())
}
