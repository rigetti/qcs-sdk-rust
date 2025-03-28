//! This module provides bindings to translate programs or fetching Quil-T calibrations
//! from the QCS API.

use std::{collections::HashMap, time::Duration};

use pyo3::{exceptions::PyValueError, PyErr};
use qcs_api_client_grpc::{
    models::controller::EncryptedControllerJob,
    services::translation::{
        translate_quil_to_encrypted_controller_job_request::NumShots,
        translation_options::{self, TranslationBackend},
        BackendV1Options, BackendV2Options, GetQuantumProcessorQuilCalibrationProgramRequest,
        TranslateQuilToEncryptedControllerJobRequest, TranslationOptions as ApiTranslationOptions,
    },
};
use tokio::time::error::Elapsed;
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::client::{GrpcClientError, Qcs, DEFAULT_HTTP_API_TIMEOUT};

/// Errors that can occur when making a request to translation service.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error due to gRPC client
    #[error(transparent)]
    Grpc(#[from] GrpcClientError),
    /// Error due to client timeout
    #[error("Client configured timeout exceeded")]
    ClientTimeout(#[from] Elapsed),
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        let message = value.to_string();
        match value {
            Error::Grpc(_) => PyValueError::new_err(message),
            Error::ClientTimeout(_) => PyValueError::new_err(message),
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
#[cfg_attr(
    feature = "tracing",
    instrument(skip(quil_program, client, translation_options))
)]
pub async fn translate<TO>(
    quantum_processor_id: &str,
    quil_program: &str,
    num_shots: u32,
    client: &Qcs,
    translation_options: Option<TO>,
) -> Result<EncryptedTranslationResult, Error>
where
    TO: Into<ApiTranslationOptions>,
{
    #[cfg(feature = "tracing")]
    tracing::debug!(
        %num_shots,
        "translating program for {}",
        quantum_processor_id,
    );

    let options = translation_options.map(Into::into);

    let request = TranslateQuilToEncryptedControllerJobRequest {
        quantum_processor_id: quantum_processor_id.to_owned(),
        num_shots: Some(NumShots::NumShotsValue(num_shots)),
        quil_program: quil_program.to_owned(),
        options,
    };

    let response = client
        .get_translation_client()
        .map_err(GrpcClientError::from)?
        .translate_quil_to_encrypted_controller_job(request)
        .await
        .map_err(GrpcClientError::from)?
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

/// Query the QCS API for Quil-T calibrations.
/// If `None`, the default `timeout` used is 10 seconds.
pub async fn get_quilt_calibrations(
    quantum_processor_id: String,
    client: &Qcs,
    timeout: Option<Duration>,
) -> Result<String, Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("getting Quil-T calibrations for {}", quantum_processor_id);

    let timeout = timeout.unwrap_or(DEFAULT_HTTP_API_TIMEOUT);

    let mut translation_client = client
        .get_translation_client()
        .map_err(GrpcClientError::from)?;

    tokio::time::timeout(timeout, async move {
        Ok(translation_client
            .get_quantum_processor_quil_calibration_program(
                GetQuantumProcessorQuilCalibrationProgramRequest {
                    quantum_processor_id,
                },
            )
            .await
            .map_err(GrpcClientError::from)?
            .into_inner()
            .quil_calibration_program)
    })
    .await?
}

/// The error returned when a specific Translation backend is expected to be set, but it is not.
#[allow(clippy::module_name_repetitions)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
pub enum TranslationBackendMismatch {
    /// Expected Translation backend V1 to be set, but got V2.
    #[error("tried to set an option for Translation V1 using a different backend")]
    V1,
    /// Expected Translation backend V2 to be set, but got V1.
    #[error("tried to set an option for Translation V2 using a different backend")]
    V2,
}

/// Options available for Quil program translation.
///
/// This wraps [`ApiTranslationOptions`] in order to improve the user experience,
/// because the structs auto-generated by `prost` can be clumsy to use directly.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Default)]
pub struct TranslationOptions {
    inner: ApiTranslationOptions,
}

impl TranslationOptions {
    /// Get the backend used for translation
    #[must_use]
    pub fn backend(&self) -> Option<&TranslationBackend> {
        self.inner.translation_backend.as_ref()
    }

    /// Get a mutable reference to the backend used for translation.
    #[must_use]
    pub fn backend_mut(&mut self) -> Option<&mut TranslationBackend> {
        self.inner.translation_backend.as_mut()
    }

    /// Use the first-generation translation backend available on QCS since 2018.
    pub fn with_backend_v1(&mut self) -> &mut BackendV1Options {
        let backend = &mut self.inner.translation_backend;
        if let Some(TranslationBackend::V1(options)) = backend {
            return options;
        }

        *backend = Some(TranslationBackend::V1(BackendV1Options::default()));
        let Some(TranslationBackend::V1(options)) = backend.as_mut() else {
            unreachable!("backend was just set to V1")
        };
        options
    }

    /// Use the second-generation translation backend available on QCS since 2023
    pub fn with_backend_v2(&mut self) -> &mut BackendV2Options {
        let backend = &mut self.inner.translation_backend;
        if let Some(TranslationBackend::V2(options)) = backend {
            return options;
        }

        *backend = Some(TranslationBackend::V2(BackendV2Options::default()));
        let Some(TranslationBackend::V2(options)) = backend.as_mut() else {
            unreachable!("backend was just set to V2")
        };
        options
    }

    #[allow(dead_code)]
    fn ensure_backend_v1(&mut self) -> Result<&mut BackendV1Options, TranslationBackendMismatch> {
        if matches!(self.backend(), None | Some(TranslationBackend::V1(_))) {
            Ok(self.with_backend_v1())
        } else {
            Err(TranslationBackendMismatch::V1)
        }
    }

    fn ensure_backend_v2(&mut self) -> Result<&mut BackendV2Options, TranslationBackendMismatch> {
        if matches!(self.backend(), None | Some(TranslationBackend::V2(_))) {
            Ok(self.with_backend_v2())
        } else {
            Err(TranslationBackendMismatch::V2)
        }
    }

    /// If `false`, default calibrations will not be prepended to the translated program.
    ///
    /// # Errors
    ///
    /// This will return an error if the translation backend is set to something other than V2.
    pub fn v2_prepend_default_calibrations(
        &mut self,
        prepend: bool,
    ) -> Result<&mut Self, TranslationBackendMismatch> {
        self.ensure_backend_v2()?.prepend_default_calibrations = Some(prepend);
        Ok(self)
    }

    /// Set a passive reset delay in seconds.
    ///
    /// # Errors
    ///
    /// This will return an error if the translation backend is set to something other than V2.
    pub fn v2_passive_reset_delay_seconds(
        &mut self,
        delay: f64,
    ) -> Result<&mut Self, TranslationBackendMismatch> {
        self.ensure_backend_v2()?.passive_reset_delay_seconds = Some(delay);
        Ok(self)
    }

    /// Request that the translation backend does not insert runtime memory access checks. Only
    /// available to certain users.
    ///
    /// # Errors
    ///
    /// This will return an error if the translation backend is set to something other than V2.
    pub fn v2_allow_unchecked_pointer_arithmetic(
        &mut self,
        allow: bool,
    ) -> Result<&mut Self, TranslationBackendMismatch> {
        self.ensure_backend_v2()?.allow_unchecked_pointer_arithmetic = Some(allow);
        Ok(self)
    }

    /// Request that the translation backend allow `DEFFRAME`s differing from the Rigetti defaults.
    /// Only available to certain users. If disallowed, only `INITIAL-FREQUENCY` and/or `CHANNEL-DELAY`
    /// may differ.
    ///
    /// # Errors
    ///
    /// This will return an error if the translation backend is set to something other than V2.
    pub fn v2_allow_frame_redefinition(
        &mut self,
        allow: bool,
    ) -> Result<&mut Self, TranslationBackendMismatch> {
        self.ensure_backend_v2()?.allow_frame_redefinition = Some(allow);
        Ok(self)
    }

    /// Set the Q-CTRL option for compiling the program through Q-CTRL's API prior to translation.
    ///
    /// Generally, the client should set this option to `QCtrl::default()`, as options are
    /// specially authorized and not generally available to the client.
    pub fn q_ctrl(&mut self, q_ctrl: translation_options::QCtrl) -> &mut Self {
        self.inner.q_ctrl = Some(q_ctrl);
        self
    }
}

impl From<TranslationOptions> for ApiTranslationOptions {
    fn from(options: TranslationOptions) -> Self {
        options.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_new_options_does_not_fail() {
        let mut options = TranslationOptions::default();
        options.v2_allow_frame_redefinition(true).unwrap();
    }
}
