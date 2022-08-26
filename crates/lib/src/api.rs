use log::warn;
use qcs_api::models::TranslateNativeQuilToEncryptedBinaryResponse;

use crate::{
    configuration::Configuration,
    qpu::{
        self, engagement,
        rpcq::Client,
        runner::{self, GetExecutionResultsResponse, JobId},
        translation,
    },
};
use std::{collections::HashMap, convert::TryFrom, sync::Mutex};

pub async fn translate(
    program: &str,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslateNativeQuilToEncryptedBinaryResponse, translation::Error> {
    translation::translate(
        qpu::RewrittenQuil(program.to_string()),
        shots,
        quantum_processor_id,
        config,
    )
    .await
}

pub async fn submit(
    program: &str,
    patch_values: HashMap<Box<str>, Vec<f64>>,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<String, runner::Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| runner::Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            runner::Error::Qpu(format!("Unable to connect to QPU via RPCQ: {:?}", e))
        })
        .map(Mutex::new)?;

    let client = rpcq_client.lock().unwrap();
    let job_id = runner::submit(program, &patch_values, &client)?;

    Ok(job_id.0)
}

pub async fn retrieve_results(
    job_id: &str,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<GetExecutionResultsResponse, runner::Error> {
    let engagement = engagement::get(String::from(quantum_processor_id), config)
        .await
        .map_err(|e| runner::Error::Qpu(format!("Unable to get engagement: {:?}", e)))?;
    let rpcq_client = Client::try_from(&engagement)
        .map_err(|e| {
            warn!("Unable to connect to QPU via RPCQ: {:?}", e);
            runner::Error::Qpu(format!("Unable to connect to QPU via RPCQ: {:?}", e))
        })
        .map(Mutex::new)?;

    let client = rpcq_client.lock().unwrap();
    runner::retrieve_results(JobId(job_id.to_string()), &client)
}
