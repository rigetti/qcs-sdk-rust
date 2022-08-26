use log::warn;
use qcs_api::models::TranslateNativeQuilToEncryptedBinaryResponse;

use crate::{
    configuration::Configuration,
    qpu::{
        self, engagement, quilc,
        rpcq::Client,
        runner::{self, GetExecutionResultsResponse, JobId},
        translation,
    },
};
use std::{collections::HashMap, convert::TryFrom, sync::Mutex};

/// Uses quilc to convert a Quil program to native Quil
pub async fn compile(
    quil: &str,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<String, Box<dyn std::error::Error>> {
    let isa = qpu::get_isa(quantum_processor_id, config).await.map_err(|e| e.to_string())?;
    quilc::compile_program(quil, isa, config).map_err(|e| e.into()).map(|p| p.0)
}

/// Translates a native Quil program into an executable
pub async fn translate(
    native_quil: &str,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslateNativeQuilToEncryptedBinaryResponse, translation::Error> {
    translation::translate(
        qpu::RewrittenQuil(native_quil.to_string()),
        shots,
        quantum_processor_id,
        config,
    )
    .await
}

/// Submits an executable `program` to be run on the specified QPU
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

/// Fetches results for the corresponding job
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
