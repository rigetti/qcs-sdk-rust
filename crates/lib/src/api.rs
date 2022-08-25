use log::warn;

use crate::{
    configuration::Configuration,
    qpu::{engagement, rpcq::Client, runner::{self, GetExecutionResultsResponse, JobId, Error}},
};
use std::{collections::HashMap, convert::TryFrom, sync::Mutex};

/// documentation
pub async fn submit(
    program: &str,
    patch_values: HashMap<Box<str>, Vec<f64>>,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<String, Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            Error::Qpu(format!("Unable to connect to QPU via RPCQ: {:?}", e))
        })
        .map(Mutex::new)?;

    let job_id: runner::JobId;
    {
        let c = rpcq_client.lock().unwrap();
        job_id = runner::submit(program, &patch_values, &c)?;
    }

    Ok(job_id.0)
}

pub async fn retrieve_results(
    job_id: &str,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<GetExecutionResultsResponse, Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            Error::Qpu(format!("Unable to connect to QPU via RPCQ: {:?}", e))
        })
        .map(Mutex::new)?;

    let c = rpcq_client.lock().unwrap();
    runner::retrieve_results(JobId(job_id.to_string()), &c)
}
