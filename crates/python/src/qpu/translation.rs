//! Translating programs.
use std::{collections::HashMap, time::Duration};

use pyo3::{exceptions::PyRuntimeError, pyclass, pyfunction, types::PyString, Py, PyResult};
use qcs::qpu::client::GrpcClientError;
use qcs_api_client_openapi::models::GetQuiltCalibrationsResponse;
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct, py_wrap_error, wrap_error, ToPythonError,
};

use crate::py_sync::py_function_sync_async;

use super::client::PyQcsClient;

create_init_submodule! {
    classes: [
        PyQuiltCalibrations,
        PyTranslationResult
    ],
    errors: [
        GetQuiltCalibrationsError,
        TranslationError
    ],
    funcs: [
        py_get_quilt_calibrations,
        py_get_quilt_calibrations_async,
        py_translate,
        py_translate_async
    ],
}

py_wrap_data_struct! {
    PyQuiltCalibrations(GetQuiltCalibrationsResponse) as "QuiltCalibrations" {
        quilt: String => Py<PyString>,
        settings_timestamp: Option<String> => Option<Py<PyString>>
    }
}

wrap_error!(RustGetQuiltCalibrationsError(
    qcs::qpu::translation::GetQuiltCalibrationsError
));
py_wrap_error!(
    translation,
    RustGetQuiltCalibrationsError,
    GetQuiltCalibrationsError,
    PyRuntimeError
);

py_function_sync_async! {
    /// Query the QCS API for Quil-T calibrations.
    /// If `None`, the default `timeout` used is 10 seconds.
    #[pyfunction(client = "None", timeout = "None")]
    async fn get_quilt_calibrations(
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
        timeout: Option<f64>,
    ) -> PyResult<PyQuiltCalibrations> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let timeout = timeout.map(Duration::from_secs_f64);
        qcs::qpu::translation::get_quilt_calibrations(&quantum_processor_id, &client, timeout)
            .await
            .map(PyQuiltCalibrations::from)
            .map_err(RustGetQuiltCalibrationsError::from)
            .map_err(RustGetQuiltCalibrationsError::to_py_err)
    }
}

/// Errors that can happen during translation
#[derive(Debug, thiserror::Error)]
pub enum RustTranslationError {
    /// The program could not be translated
    #[error("Could not translate quil: {0}")]
    Translate(#[from] GrpcClientError),
    /// The result of translation could not be deserialized
    #[error("Could not serialize translation result: {0}")]
    Serialize(#[from] serde_json::Error),
}

py_wrap_error!(
    translation,
    RustTranslationError,
    TranslationError,
    PyRuntimeError
);

/// The result of a call to [`translate`] which provides information about the
/// translated program.
#[pyclass]
#[pyo3(name = "TranslationResult")]
pub struct PyTranslationResult {
    /// The translated program.
    #[pyo3(get)]
    pub program: String,

    /// The memory locations used for readout.
    #[pyo3(get)]
    pub ro_sources: Option<HashMap<String, String>>,
}

#[pyclass]
struct V1Options {}

#[pyclass]
struct V2Options {}

#[derive(Debug, Clone)]
#[pyclass]
enum TranslationBackend {
    V1,
    V2,
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct TranslationOptions {
    backend: Option<TranslationBackend>,
}

impl From<TranslationOptions> for qcs::qpu::translation::TranslationOptions {
    fn from(value: TranslationOptions) -> Self {
        let backend = value.backend.map(|backend| match backend {
            TranslationBackend::V1 => qcs::qpu::translation::TranslationBackend::V1(
                qcs::qpu::translation::V1TranslationBackendOptions {},
            ),
            TranslationBackend::V2 => qcs::qpu::translation::TranslationBackend::V2(
                qcs::qpu::translation::V2TranslationBackendOptions {},
            ),
        });
        Self { backend }
    }
}

py_function_sync_async! {
    #[pyfunction(client = "None")]
    /// Translates a native Quil program into an executable
    ///
    /// # Errors
    ///
    /// Returns a [`TranslationError`] if translation fails.
    async fn translate(
        native_quil: String,
        num_shots: u32,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
        translation_options: Option<TranslationOptions>,
    ) -> PyResult<PyTranslationResult> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let translation_options = translation_options.map(Into::into);
        let result =
            qcs::qpu::translation::translate(&quantum_processor_id, &native_quil, num_shots, translation_options, &client)
                .await
                .map_err(RustTranslationError::from)
                .map_err(RustTranslationError::to_py_err)?;

        let program = serde_json::to_string(&result.job)
            .map_err(RustTranslationError::from)
            .map_err(RustTranslationError::to_py_err)?;

        Ok(PyTranslationResult {
            program,
            ro_sources: Some(result.readout_map),
        })
    }
}
