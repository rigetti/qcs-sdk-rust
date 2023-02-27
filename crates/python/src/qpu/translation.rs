//! Translating programs.
use std::collections::HashMap;

use pyo3::{exceptions::PyRuntimeError, pyclass, pyfunction, PyResult};
use qcs::qpu::client::GrpcClientError;
use rigetti_pyo3::{create_init_submodule, py_wrap_error, ToPythonError};

use crate::py_sync::py_function_sync_async;

use super::client::PyQcsClient;

create_init_submodule! {
    classes: [
        PyTranslationResult
    ],
    errors: [
        PyTranslationError
    ],
    funcs: [
        py_translate,
        py_translate_async
    ],
}

/// Errors that can happen during translation
#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    /// The program could not be translated
    #[error("Could not translate quil: {0}")]
    Translate(#[from] GrpcClientError),
    /// The result of translation could not be deserialized
    #[error("Could not serialize translation result: {0}")]
    Serialize(#[from] serde_json::Error),
}

py_wrap_error!(
    translation,
    TranslationError,
    PyTranslationError,
    PyRuntimeError
);

/// The result of a call to [`translate`] which provides information about the
/// translated program.
// #[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[pyclass]
#[pyo3(name = "TranslationResult")]
pub struct PyTranslationResult {
    /// The translated program.
    pub program: String,

    /// The memory locations used for readout.
    pub ro_sources: Option<HashMap<String, String>>,
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
    ) -> PyResult<PyTranslationResult> {
        let client = PyQcsClient::get_or_create_client(client).await?;
        let result =
            qcs::qpu::translation::translate(&quantum_processor_id, &native_quil, num_shots, &client)
                .await
                .map_err(TranslationError::from)
                .map_err(TranslationError::to_py_err)?;

        let program = serde_json::to_string(&result.job)
            .map_err(TranslationError::from)
            .map_err(TranslationError::to_py_err)?;

        Ok(PyTranslationResult {
            program,
            ro_sources: Some(result.readout_map),
        })
    }
}
