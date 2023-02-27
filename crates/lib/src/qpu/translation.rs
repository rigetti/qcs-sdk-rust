//! Translation
use std::{collections::HashMap, time::Duration};

use qcs_api_client_grpc::{
    models::controller::EncryptedControllerJob,
    services::translation::{
        translate_quil_to_encrypted_controller_job_request::NumShots,
        TranslateQuilToEncryptedControllerJobRequest,
    },
};
use qcs_api_client_openapi::{
    apis::{translation_api, Error as OpenAPIError},
    models::GetQuiltCalibrationsResponse,
};
use tokio::time::error::Elapsed;

use super::client::{GrpcClientError, Qcs, DEFAULT_HTTP_API_TIMEOUT};

/// An encrypted and translated program, along with readout_map
/// to map job readout_data back to program-declared variables.
#[derive(Debug)]
pub struct EncryptedTranslationResult {
    /// The encrypted, translated program.
    pub job: EncryptedControllerJob,

    /// A mapping of translated program variable names,
    /// which will be returned from job execution,
    /// back to the original pre-translation user-defined
    /// program variable names.
    pub readout_map: HashMap<String, String>,
}

/// Translate a program, returning an encrypted and translated program.
pub async fn translate(
    quantum_processor_id: &str,
    quil_program: &str,
    num_shots: u32,
    client: &Qcs,
) -> Result<EncryptedTranslationResult, GrpcClientError> {
    let request = TranslateQuilToEncryptedControllerJobRequest {
        quantum_processor_id: Some(quantum_processor_id.to_owned()),
        num_shots: Some(NumShots::NumShotsValue(num_shots)),
        quil_program: Some(quil_program.to_owned()),
    };

    let response = client
        .get_translation_client()?
        .translate_quil_to_encrypted_controller_job(request)
        .await?
        .into_inner();

    Ok(EncryptedTranslationResult {
        job: response
            .job
            .ok_or_else(|| GrpcClientError::ResponseEmpty("Encrypted Job".into()))?,
        readout_map: response
            .metadata
            .ok_or_else(|| GrpcClientError::ResponseEmpty("Job Metadata".into()))?
            .readout_mappings,
    })
}

/// API Errors encountered when trying to get Quil-T calibrations.
#[derive(Debug, thiserror::Error)]
pub enum GetQuiltCalibrationsError {
    /// Failed the http call
    #[error("Failed to get Quil-T calibrations via API: {0}")]
    ApiError(#[from] OpenAPIError<translation_api::GetQuiltCalibrationsError>),

    /// API call did not finish before timeout
    #[error("API call did not finish before timeout.")]
    TimeoutError(#[from] Elapsed),
}

/// Query the QCS API for Quil-T calibrations.
/// If `None`, the default `timeout` used is 10 seconds.
pub async fn get_quilt_calibrations(
    quantum_processor_id: &str,
    client: &Qcs,
    timeout: Option<Duration>,
) -> Result<GetQuiltCalibrationsResponse, GetQuiltCalibrationsError> {
    let timeout = timeout.unwrap_or(DEFAULT_HTTP_API_TIMEOUT);

    tokio::time::timeout(timeout, async move {
        Ok(translation_api::get_quilt_calibrations(
            &client.get_openapi_client(),
            quantum_processor_id,
        )
        .await?)
    })
    .await?
}
