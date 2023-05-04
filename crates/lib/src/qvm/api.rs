//! This module provides functions to for makings calls to the QVM.
use std::borrow::Cow;
use std::str::FromStr;

use qcs_api_client_common::ClientConfiguration;
use quil_rs::{
    instruction::{ArithmeticOperand, Instruction, MemoryReference, Move},
    Program,
};

use super::{Error, MultishotMeasureRequest, QvmResultData};
use crate::{
    executable::Parameters,
    qvm::{MultishotRequest, MultishotResponse},
};

/// Execute a program on the QVM.
pub async fn run(
    quil: &str,
    shots: u16,
    readouts: &[Cow<'_, str>],
    params: &Parameters,
    config: &ClientConfiguration,
) -> Result<QvmResultData, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        %shots,
        ?readouts,
        ?params,
        "preparing program to run on QVM"
    );

    let program = Program::from_str(quil).map_err(Error::Parsing)?;

    run_program(&program, shots, readouts, params, config).await
}

pub async fn run_and_measure(
    shots: u16,
    qubits: Vec<u64>,
    measurement_noise: Option<(f64, f64, f64)>,
    seed: Option<i64>,
) -> Result<Vec<Vec<i64>>, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        %shots,
        ?readouts,
        ?params,
        "preparing program to run on QVM"
    );

    let program = Program::from_str(quil).map_err(Error::Parsing)?;

    run_and_measure_program(&program, shots, readouts, params, config).await
}

pub(crate) async fn run_program(
    program: &Program,
    shots: u16,
    readouts: &[Cow<'_, str>],
    params: &Parameters,
    config: &ClientConfiguration,
) -> Result<QvmResultData, Error> {
    if shots == 0 {
        return Err(Error::ShotsMustBePositive);
    }

    // Create a clone of the program so MOVE statements can be prepended to it
    let mut program = program.clone();

    for (name, values) in params {
        match program.memory_regions.get(name.as_ref()) {
            Some(region) => {
                if region.size.length != values.len() as u64 {
                    return Err(Error::RegionSizeMismatch {
                        name: name.clone(),
                        declared: region.size.length,
                        parameters: values.len(),
                    });
                }
            }
            None => {
                return Err(Error::RegionNotFound { name: name.clone() });
            }
        }
        for (index, value) in values.iter().enumerate() {
            program.instructions.insert(
                0,
                Instruction::Move(Move {
                    destination: ArithmeticOperand::MemoryReference(MemoryReference {
                        name: name.to_string(),
                        index: index as u64,
                    }),
                    source: ArithmeticOperand::LiteralReal(*value),
                }),
            );
        }
    }
    execute(&program, shots, readouts, config).await
}

pub(crate) async fn run_and_measure_program(
    program: &Program,
    shots: u16,
    qubits: Vec<u64>,
    measurement_noise: Option<(f64, f64, f64)>,
    seed: Option<i64>,
    config: &ClientConfiguration,
) -> Result<Vec<Vec<i64>>, Error> {
    let request = MultishotMeasureRequest::new(
        &program.to_string(true),
        shots,
        qubits,
        measurement_noise,
        seed,
    );

    let client = reqwest::Client::new();
    let response = client
        .post(config.qvm_url())
        .json(&request)
        .send()
        .await
        .map_err(|source| Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        })?;

    match response.json::<MultishotResponse>().await {
        Ok(MultishotResponse::Success(response)) => Ok(response.into()),
        Ok(MultishotResponse::Failure(response)) => Err(Error::Qvm {
            message: response.status,
        }),
        Err(source) => Err(Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        }),
    }
}

pub(crate) async fn execute(
    program: &Program,
    shots: u16,
    readouts: &[Cow<'_, str>],
    config: &ClientConfiguration,
) -> Result<QvmResultData, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        %shots,
        ?readouts,
        "executing program on QVM"
    );
    let request = MultishotRequest::new(&program.to_string(true), shots, readouts);

    let client = reqwest::Client::new();
    let response = client
        .post(config.qvm_url())
        .json(&request)
        .send()
        .await
        .map_err(|source| Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        })?;

    match response.json::<MultishotResponse>().await {
        Err(source) => Err(Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        }),
        Ok(MultishotResponse::Success(response)) => {
            Ok(QvmResultData::from_memory_map(response.registers))
        }
        Ok(MultishotResponse::Failure(response)) => Err(Error::Qvm {
            message: response.status,
        }),
    }
}

/// Fetch the version information from the running QVM server.
pub async fn get_version_info(config: &ClientConfiguration) -> Result<String, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("requesting qvm version information");
    let client = reqwest::Client::new();
    let params = vec![("type", "version")];
    client
        .post(config.qvm_url())
        .json(&params)
        .send()
        .await
        .map_err(|source| Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        })?
        .text()
        .await
        .map_err(|source| Error::QvmCommunication {
            qvm_url: config.qvm_url().into(),
            source,
        })
}
