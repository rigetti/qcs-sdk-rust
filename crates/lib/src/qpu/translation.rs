//! Translation
use std::collections::HashMap;

use qcs_api_client_grpc::{
    models::controller::EncryptedControllerJob,
    services::translation::{
        translate_quil_to_encrypted_controller_job_request::NumShots,
        TranslateQuilToEncryptedControllerJobRequest,
    },
};

use super::client::{GrpcClientError, Qcs};

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
