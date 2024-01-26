//! Translating programs.
use std::{collections::HashMap, time::Duration};

use pyo3::{
    exceptions::PyRuntimeError, pyclass, pyfunction, pymethods, types::PyString, Py, PyResult,
};
use qcs::client::GrpcClientError;
use qcs::qpu::translation::TranslationOptions;
use qcs_api_client_grpc::services::translation::translation_options::TranslationBackend as ApiTranslationBackend;
use qcs_api_client_openapi::models::GetQuiltCalibrationsResponse;
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct, py_wrap_error, py_wrap_simple_enum, wrap_error,
    ToPythonError,
};

use crate::py_sync::py_function_sync_async;

use crate::client::PyQcsClient;

create_init_submodule! {
    classes: [
        PyQuiltCalibrations,
        PyTranslationOptions,
        PyTranslationResult,
        PyTranslationBackend
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
    #[pyfunction]
    #[pyo3(signature = (quantum_processor_id, client = None, timeout = None))]
    async fn get_quilt_calibrations(
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
        timeout: Option<f64>,
    ) -> PyResult<PyQuiltCalibrations> {
        let client = PyQcsClient::get_or_create_client(client).await;
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

#[derive(Copy, Clone)]
pub enum TranslationBackend {
    V1,
    V2,
}

py_wrap_simple_enum! {
    PyTranslationBackend(TranslationBackend) as "TranslationBackend" {
        V1,
        V2
    }
}

#[derive(Clone, Default)]
#[pyclass(name = "TranslationOptions")]
pub struct PyTranslationOptions(TranslationOptions);

impl PyTranslationOptions {
    pub fn as_inner(&self) -> &TranslationOptions {
        &self.0
    }
}

#[pymethods]
impl PyTranslationOptions {
    #[new]
    fn __new__() -> PyResult<Self> {
        Ok(Self(Default::default()))
    }

    #[getter]
    fn backend(&self) -> Option<PyTranslationBackend> {
        self.0.backend().map(|b| match b {
            ApiTranslationBackend::V1(_) => PyTranslationBackend::V1,
            ApiTranslationBackend::V2(_) => PyTranslationBackend::V2,
        })
    }

    fn use_backend_v1(&mut self) {
        self.0.with_backend_v1();
    }

    fn use_backend_v2(&mut self) {
        self.0.with_backend_v2();
    }

    #[staticmethod]
    fn v1() -> Self {
        let mut builder = TranslationOptions::default();
        builder.with_backend_v1();
        Self(builder)
    }

    #[staticmethod]
    #[pyo3(signature = (
        *,
        prepend_default_calibrations=None,
        passive_reset_delay_seconds=None,
        allow_unchecked_pointer_arithmetic=None,
        allow_frame_redefinition=None
    ))]
    fn v2(
        prepend_default_calibrations: Option<bool>,
        passive_reset_delay_seconds: Option<f64>,
        allow_unchecked_pointer_arithmetic: Option<bool>,
        allow_frame_redefinition: Option<bool>,
    ) -> Self {
        let mut builder = TranslationOptions::default();
        builder.with_backend_v2();
        if let Some(prepend) = prepend_default_calibrations {
            builder
                .v2_prepend_default_calibrations(prepend)
                .expect("using the correct backend");
        }
        if let Some(delay) = passive_reset_delay_seconds {
            builder
                .v2_passive_reset_delay_seconds(delay)
                .expect("using the correct backend");
        }
        if let Some(allow) = allow_unchecked_pointer_arithmetic {
            builder
                .v2_allow_unchecked_pointer_arithmetic(allow)
                .expect("using the correct backend");
        }
        if let Some(allow) = allow_frame_redefinition {
            builder
                .v2_allow_frame_redefinition(allow)
                .expect("using the correct backend");
        }
        Self(builder)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

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

py_function_sync_async! {
    /// Translates a native Quil program into an executable
    ///
    /// # Errors
    ///
    /// Returns a [`TranslationError`] if translation fails.
    #[pyfunction]
    #[pyo3(signature = (native_quil, num_shots, quantum_processor_id, client = None, translation_options = None))]
    async fn translate(
        native_quil: String,
        num_shots: u32,
        quantum_processor_id: String,
        client: Option<PyQcsClient>,
        translation_options: Option<PyTranslationOptions>,
    ) -> PyResult<PyTranslationResult> {
        let client = PyQcsClient::get_or_create_client(client).await;
        let translation_options = translation_options.map(|opts| opts.as_inner().clone());
        let result =
            qcs::qpu::translation::translate(&quantum_processor_id, &native_quil, num_shots, &client, translation_options)
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
