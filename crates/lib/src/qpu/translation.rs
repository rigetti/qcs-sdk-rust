//! This module provides bindings to translate programs or fetching Quil-T calibrations
//! from the QCS API.

use std::{collections::HashMap, time::Duration};

use qcs_api_client_grpc::{
    models::controller::EncryptedControllerJob,
    services::translation::{
        translate_quil_to_encrypted_controller_job_request::NumShots,
        translation_options::TranslationBackend as RdmTranslationBackend,
        BackendV1Options as RdmBackendV1Options, BackendV2Options as RdmBackendV2Options,
        TranslateQuilToEncryptedControllerJobRequest, TranslationOptions as RdmTranslationOptions,
    },
};
use qcs_api_client_openapi::{
    apis::{translation_api, Error as OpenAPIError},
    models::GetQuiltCalibrationsResponse,
};
use tokio::time::error::Elapsed;

use super::client::{GrpcClientError, Qcs, DEFAULT_HTTP_API_TIMEOUT};

/// Options to use when translating with the V1 translation backend
#[derive(Debug, Clone, Copy)]
pub struct V1TranslationBackendOptions {}

impl From<V1TranslationBackendOptions> for RdmBackendV1Options {
    fn from(_options: V1TranslationBackendOptions) -> Self {
        RdmBackendV1Options {}
    }
}

/// Options to use when translating with the V2 translation backend
#[derive(Debug, Clone, Copy)]
pub struct V2TranslationBackendOptions {}

impl From<V2TranslationBackendOptions> for RdmBackendV2Options {
    fn from(_options: V2TranslationBackendOptions) -> Self {
        RdmBackendV2Options {}
    }
}

/// The backend to use for translation
#[derive(Debug, Clone, Copy)]
pub enum TranslationBackend {
    /// V1 is the "legacy" translation backend
    V1(V1TranslationBackendOptions),
    /// V2 is the next-gen backend supporting control-flow
    V2(V2TranslationBackendOptions),
}

impl From<TranslationBackend> for RdmTranslationBackend {
    fn from(backend: TranslationBackend) -> Self {
        match backend {
            TranslationBackend::V1(options) => RdmTranslationBackend::V1(options.into()),
            TranslationBackend::V2(options) => RdmTranslationBackend::V2(options.into()),
        }
    }
}

/// The options to use in translation
#[derive(Debug, Clone, Copy)]
pub struct TranslationOptions {
    /// The backend options to use in translation
    pub backend: Option<TranslationBackend>,
}

impl From<TranslationOptions> for RdmTranslationOptions {
    fn from(options: TranslationOptions) -> Self {
        RdmTranslationOptions {
            translation_backend: options.backend.map(Into::into),
        }
    }
}

/// An encrypted and translated program, along with `readout_map`
/// to map job `readout_data` back to program-declared variables.
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
    translation_options: Option<TranslationOptions>,
    client: &Qcs,
) -> Result<EncryptedTranslationResult, GrpcClientError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        %num_shots,
        "translating program for {}",
        quantum_processor_id,
    );

    let request = TranslateQuilToEncryptedControllerJobRequest {
        quantum_processor_id: quantum_processor_id.to_owned(),
        num_shots: Some(NumShots::NumShotsValue(num_shots)),
        quil_program: quil_program.to_owned(),
        options: translation_options.map(Into::into),
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
    #[cfg(feature = "tracing")]
    tracing::debug!("getting Quil-T calibrations for {}", quantum_processor_id);

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
