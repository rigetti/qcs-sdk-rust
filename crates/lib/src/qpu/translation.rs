use reqwest::StatusCode;

use qcs_api::apis::translation_api as translation;
use qcs_api::apis::translation_api::TranslateNativeQuilToEncryptedBinaryError;
use qcs_api::apis::Error as UntypedApiError;
use qcs_api::models::{
    Error as QcsError, TranslateNativeQuilToEncryptedBinaryRequest,
    TranslateNativeQuilToEncryptedBinaryResponse,
};

use crate::configuration::Configuration;
use crate::qpu::rewrite_arithmetic::RewrittenQuil;

type ApiError = UntypedApiError<TranslateNativeQuilToEncryptedBinaryError>;

pub(crate) async fn translate(
    quil: RewrittenQuil,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslateNativeQuilToEncryptedBinaryResponse, Error> {
    let translation_request = TranslateNativeQuilToEncryptedBinaryRequest {
        num_shots: shots.into(),
        quil: quil.into(),
        settings_timestamp: None,
    };
    translation::translate_native_quil_to_encrypted_binary(
        config.as_ref(),
        quantum_processor_id,
        translation_request,
    )
    .await
    .map_err(Error::from)
}

/// Errors that can occur during Translation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Provided program could not be translated: {}", .0.message)]
    ProgramIssue(QcsError),
    #[error("Problem connecting to QCS")]
    Connection(#[source] ApiError),
    #[error("Serialization failed, this is likely a bug in this library")]
    Serialization(#[from] serde_json::Error),
    #[error("Unauthorized request")]
    Unauthorized,
    #[error("An unknown error occurred, this is likely a bug in this library: {0}")]
    Unknown(String),
}

impl From<ApiError> for Error {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::Reqwest(_) | ApiError::Io(_) => Self::Connection(error),
            ApiError::Serde(inner) => inner.into(),
            ApiError::ResponseError(inner) => {
                if matches!(
                    inner.status,
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
                ) {
                    Self::Unauthorized
                } else {
                    match inner.entity {
                        Some(TranslateNativeQuilToEncryptedBinaryError::Status400(detail))
                            if inner.status == StatusCode::BAD_REQUEST =>
                        {
                            Self::ProgramIssue(detail)
                        }
                        _ => Self::Unknown(inner.content),
                    }
                }
            }
        }
    }
}
