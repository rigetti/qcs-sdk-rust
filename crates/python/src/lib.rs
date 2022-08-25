use qcs::qpu::rpcq::Client;
use qcs::Executable;
use std::collections::HashMap;

use pyo3::prelude::*;

#[pyfunction]
fn submit(program: &str, patch_values: HashMap<String, f64>) -> PyResult<String> {
    let engagement = engagement::get(String::from(self.quantum_processor_id), config).await?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            Error::QcsCommunication
        })
        .map(Mutex::new)?;
    let executable = Executable::from_quil(program);

    Ok("".to_string())
}

#[pymodule]
fn qcs_(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    Ok(())
}
