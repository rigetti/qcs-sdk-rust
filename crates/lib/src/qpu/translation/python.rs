use std::collections::HashMap;
use std::time::Duration;

use futures_util::TryFutureExt;
use opentelemetry::trace::FutureExt;
use prost::Message;
use pyo3::types::PyBytes;
use pyo3::{exceptions::PyValueError, prelude::*, PyErr};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{
    gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pyfunction, gen_stub_pymethods,
};

use qcs_api_client_grpc::services::translation::translation_options::{self, Riverlane};
use qcs_api_client_grpc::services::translation::{
    translation_options::TranslationBackend as ApiTranslationBackend,
    TranslationOptions as ApiTranslationOptions,
};

use crate::client::Qcs;
use crate::python::{errors, impl_repr, py_function_sync_async};
use crate::qpu::translation::{get_quilt_calibrations, Error, TranslationOptions};

#[pymodule]
#[pyo3(name = "translation", module = "qcs_sdk.qpu", submodule)]
pub(crate) fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    m.add(
        "TranslationError",
        py.get_type::<errors::TranslationError>(),
    )?;

    m.add_class::<PyTranslationOptions>()?;
    m.add_class::<PyTranslationResult>()?;
    m.add_class::<PyTranslationBackend>()?;
    m.add_class::<PyQCtrl>()?;
    m.add_class::<PyRiverlane>()?;

    m.add_function(wrap_pyfunction!(py_get_quilt_calibrations, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_quilt_calibrations_async, m)?)?;
    m.add_function(wrap_pyfunction!(py_translate, m)?)?;
    m.add_function(wrap_pyfunction!(py_translate_async, m)?)?;

    Ok(())
}

impl_repr!(TranslationOptions);

py_function_sync_async! {
    /// Query the QCS API for Quil-T calibrations.
    /// If `None`, the default `timeout` used is 10 seconds.
    #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.translation"))]
    #[pyfunction]
    #[pyo3(signature = (quantum_processor_id, client = None, timeout = None))]
    // TODO #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    async fn get_quilt_calibrations(
        quantum_processor_id: String,
        client: Option<Qcs>,
        timeout: Option<f64>,
    ) -> PyResult<String> {
        let client = client.unwrap_or_else(Qcs::load);
        let timeout = timeout.map(Duration::from_secs_f64);
        get_quilt_calibrations(quantum_processor_id, &client, timeout)
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error(transparent)]
    Translation(#[from] Error),
    #[error("Failed to serialize translation result: {0}")]
    Serialization(#[from] serde_json::Error),
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

#[derive(Copy, Clone, Debug)]
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass_enum)]
#[pyclass(module = "qcs_sdk.qpu.translation", name = "TranslationBackend")]
pub enum PyTranslationBackend {
    V1,
    V2,
}

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "TranslationOptions", module = "qcs_sdk.qpu.translation")]
pub(crate) struct PyTranslationOptions(TranslationOptions);

impl PyTranslationOptions {
    pub fn as_inner(&self) -> &TranslationOptions {
        &self.0
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl TranslationOptions {
    /// Get the backend used for translation
    #[pyo3(name = "backend")]
    pub fn py_backend(&self) -> Option<PyTranslationBackend> {
        self.inner.translation_backend.map(|b| match b {
            ApiTranslationBackend::V1(_) => PyTranslationBackend::V1,
            ApiTranslationBackend::V2(_) => PyTranslationBackend::V2,
        })
    }

    fn use_backend_v1(&mut self) {
        self.with_backend_v1();
    }

    fn use_backend_v2(&mut self) {
        self.with_backend_v2();
    }

    #[pyo3(signature = (q_ctrl = PyQCtrl::default()))]
    fn use_q_ctrl(&mut self, q_ctrl: PyQCtrl) {
        self.q_ctrl(*q_ctrl.as_inner());
    }

    #[pyo3(signature = (riverlane = PyRiverlane::default()))]
    fn use_riverlane(&mut self, riverlane: PyRiverlane) {
        self.riverlane(riverlane.as_inner().clone());
    }

    #[staticmethod]
    fn v1() -> Self {
        let mut builder = TranslationOptions::default();
        builder.with_backend_v1();
        builder
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

        builder
    }

    fn encode_as_protobuf<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let options: ApiTranslationOptions = self.clone().into();
        PyBytes::new(py, options.encode_to_vec().as_slice())
    }
}

#[derive(Clone, Default, Debug)]
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "QCtrl", module = "qcs_sdk.qpu.translation", frozen)]
pub struct PyQCtrl(translation_options::QCtrl);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyQCtrl {
    #[new]
    #[pyo3(signature = (fixed_layout = None))]
    fn __new__(fixed_layout: Option<bool>) -> PyResult<Self> {
        Ok(Self(translation_options::QCtrl { fixed_layout }))
    }
}

impl PyQCtrl {
    fn as_inner(&self) -> &translation_options::QCtrl {
        &self.0
    }
}

#[derive(Clone, Default, Debug)]
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Riverlane", module = "qcs_sdk.qpu.translation", frozen)]
pub struct PyRiverlane(Riverlane);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyRiverlane {
    #[new]
    #[pyo3(signature = (qeci_configuration_data, qeci_max_nanoseconds_until_read_available))]
    fn __new__(
        qeci_configuration_data: HashMap<String, Vec<u8>>,
        qeci_max_nanoseconds_until_read_available: u64,
    ) -> PyResult<Self> {
        Ok(Self(Riverlane {
            qeci_configuration_data,
            qeci_max_nanoseconds_until_read_available,
        }))
    }
}

impl PyRiverlane {
    fn as_inner(&self) -> &Riverlane {
        &self.0
    }
}

/// The result of a call to [`translate`] which provides information about the
/// translated program.
#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(
    module = "qcs_sdk.qpu.translation",
    name = "TranslationResult",
    get_all,
    frozen
)]
pub struct PyTranslationResult {
    /// The translated program.
    pub program: String,

    /// The memory locations used for readout.
    pub ro_sources: Option<HashMap<String, String>>,
}

py_function_sync_async! {
    /// Translates a native Quil program into an executable
    ///
    /// # Errors
    ///
    /// Returns a [`TranslationError`] if translation fails.
    #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.translation"))]
    #[pyfunction]
    #[pyo3(signature = (native_quil, num_shots, quantum_processor_id, client = None, translation_options = None))]
    // TODO #[pyo3_opentelemetry::pypropagate(on_context_extraction_failure="ignore")]
    async fn translate(
        native_quil: String,
        num_shots: u32,
        quantum_processor_id: String,
        client: Option<Qcs>,
        translation_options: Option<PyTranslationOptions>,
    ) -> PyResult<PyTranslationResult> {
        let client = client.unwrap_or_else(Qcs::load);
        let translation_options = translation_options.map(|opts| opts.as_inner().clone());

        let result = crate::qpu::translation::translate(&quantum_processor_id, &native_quil, num_shots, &client, translation_options).with_current_context()
                .map_err(TranslationError::from)
                .await?;

        let program = serde_json::to_string(&result.job)
                .map_err(TranslationError::from)?;

        Ok(PyTranslationResult {
            program,
            ro_sources: Some(result.readout_map),
        })
    }
}
