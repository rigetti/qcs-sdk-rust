use log::warn;

use crate::{
    configuration::Configuration,
    qpu::{engagement, rpcq::Client, runner, runner::Error, JobId},
};
use std::{collections::HashMap, convert::TryFrom, sync::Mutex};

/// documentation
pub async fn submit(
    program: &str,
    patch_values: HashMap<String, f64>,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<JobId, Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| Error::Qpu("TODO".to_string()))?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            Error::Qpu("TODO".to_string())
        })
        .map(Mutex::new)?;

    let job_id = runner::submit(program, patch_values, &rpcq_client)?;

    Ok(JobId("job-id".to_string()))
}
