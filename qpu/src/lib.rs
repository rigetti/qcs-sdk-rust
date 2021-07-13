#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use qcs_api::apis::quantum_processors_api as qpu_api;
use qcs_api::apis::translation_api as translation;
use qcs_api::apis::Error as QcsError;
use qcs_api::models;
use qcs_api::models::InstructionSetArchitecture;
use qcs_util::Configuration;

/// Run a Quil program on a real QPU
///
/// # Arguments
/// 1. `quil`: The Quil program as a string,
/// 2. `shots`: The number of times the program should run.
/// 3. `register`: The name of the register containing results that should be read out from QVM.
/// 4. `quantum_processor_id`: The name of the QPU to run on.
///
/// # Errors
/// See [`Error`] for possible error conditions.
pub async fn run_program(
    quil: &str,
    shots: u16,
    _register: &str,
    quantum_processor_id: &str,
) -> Result<models::TranslateNativeQuilToEncryptedBinaryResponse, Error> {
    let (isa, config) = get_isa(quantum_processor_id).await?;
    let native_quil = quilc::compile_program(quil, &isa, &config)?.quil;
    let translation_request = models::TranslateNativeQuilToEncryptedBinaryRequest {
        num_shots: shots.into(),
        quil: native_quil,
        settings_timestamp: None,
    };
    let exe = translation::translate_native_quil_to_encrypted_binary(
        config.as_ref(),
        quantum_processor_id,
        translation_request,
    )
    .await?;
    Ok(exe)
}

#[cfg(test)]
#[cfg(feature = "qcs_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn smoke() {
        let resp = run_program("H 0", 2, "ro", "Aspen-9").await.unwrap();
        eprintln!("{:#?}", resp);
    }
}

async fn get_isa(
    quantum_processor_id: &str,
) -> Result<(InstructionSetArchitecture, Configuration), Error> {
    let mut config = qcs_util::get_configuration().await?;
    let initial =
        qpu_api::get_instruction_set_architecture(config.as_ref(), quantum_processor_id).await;
    if let Ok(data) = initial {
        Ok((data, config))
    } else {
        config = config.refresh().await?;
        let data = qpu_api::get_instruction_set_architecture(config.as_ref(), quantum_processor_id)
            .await?;
        Ok((data, config))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error loading config.")]
    Config(#[from] qcs_util::ConfigError),
    #[error("Could not fetch details for the requested QPU")]
    QpuLookup(#[from] QcsError<qpu_api::GetInstructionSetArchitectureError>),
    #[error("Failed to compile into native quil using quilc")]
    Quilc(#[from] quilc::CompileError),
    #[error("Could not translate native quil into binary")]
    Translate(#[from] QcsError<translation::TranslateNativeQuilToEncryptedBinaryError>),
}
