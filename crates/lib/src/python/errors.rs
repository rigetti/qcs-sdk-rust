//! This module defines exceptions used (or catchable) on the Python-side,
//! along with conversions from their Rust error counterparts within this crate.
use pyo3::exceptions::PyException;
use qcs_api_client_common::configuration;

/// Create a new Python exception using the correct macro
/// based on whether the "stubs" features is active.
#[allow(unused_macro_rules)]
macro_rules! create_exception {
    ( $module:expr, $py_err: ident, $base: ty ) => {
        create_exception!($module, $py_err, $base, "");
    };
    ( $module:expr, $py_err: ident, $base: ty, $doc: expr ) => {
        ::pyo3::create_exception!($module, $py_err, $base, $doc);

        #[cfg(feature = "stubs")]
        impl ::pyo3_stub_gen::PyStubType for $py_err {
            fn type_output() -> ::pyo3_stub_gen::TypeInfo {
                ::pyo3_stub_gen::TypeInfo::locally_defined(
                    stringify!($py_err),
                    stringify!($module).into(),
                )
            }
        }

        #[cfg(feature = "stubs")]
        pyo3_stub_gen::inventory::submit! {
            pyo3_stub_gen::type_info::PyClassInfo {
                pyclass_name: stringify!($py_err),
                struct_id: std::any::TypeId::of::<$py_err>,
                getters: &[],
                setters: &[],
                module: Some(stringify!($module)),
                doc: $doc,
                bases: &[|| <$base as pyo3_stub_gen::PyStubType>::type_output()],
                has_eq: false,
                has_ord: false,
                has_hash: false,
                has_str: false,
            }
        }
    };
}

/// Create a Python exception and a conversion from its Rust type.
/// Note that the exception class must still be added to the module.
macro_rules! exception {
    ( $rust_err: ty, $module:expr, $py_err: ident, $base: ty $(, $doc: expr)? ) => {
        create_exception!( $module, $py_err, $base $(, $doc)? );

        #[doc = concat!(
            "Convert a Rust ",
            "`", stringify!($rust_err), "`",
            " into a Python ",
            "`", stringify!($py_err), "`."
        )]
        impl std::convert::From<$rust_err> for pyo3::PyErr {
            fn from(err: $rust_err) -> Self {
                $py_err::new_err(err.to_string())
            }
        }
    };
}

create_exception!(
    qcs_sdk,
    QcsSdkError,
    PyException,
    "Base exception type for errors raised by this package."
);

exception!(
    crate::Error,
    qcs_sdk,
    ExecutionError,
    QcsSdkError,
    "Errors which can occur when executing a program."
);

exception!(
    crate::RegisterMatrixConversionError,
    qcs_sdk,
    RegisterMatrixConversionError,
    QcsSdkError,
    "Error that may occur when building a `RegisterMatrix` from execution data."
);

create_exception!(
    qcs_sdk.client,
    ClientError,
    QcsSdkError,
    "Errors encountered while interacting with a QCS API client."
);

create_exception!(
    qcs_sdk.client,
    BuildClientError,
    ClientError,
    "Errors encountered while building the QCS API client configuration manually."
);

create_exception!(
    qcs_sdk.client,
    LoadClientError,
    ClientError,
    "Errors encountered while loading the QCS API client configuration from the environment configuration."
);

create_exception!(
    qcs_sdk.client,
    TokenError,
    ClientError,
    "Errors that can occur when managing authorization tokens."
);

/// These are conversions from error types in other crates to `PyErr`s of our target types.
/// These can't be implemented using `From` due to orphan rules.
impl ClientError {
    pub(crate) fn new<E: std::fmt::Display>(err: E) -> pyo3::PyErr {
        ClientError::new_err(err.to_string())
    }

    pub(crate) fn load_error(err: configuration::LoadError) -> pyo3::PyErr {
        LoadClientError::new_err(err.to_string())
    }

    pub(crate) fn builder_error(
        err: configuration::ClientConfigurationBuilderError,
    ) -> pyo3::PyErr {
        BuildClientError::new_err(err.to_string())
    }

    pub(crate) fn token_error(err: configuration::TokenError) -> pyo3::PyErr {
        TokenError::new_err(err.to_string())
    }
}

exception!(
    crate::compiler::quilc::Error,
    qcs_sdk.compiler.quilc,
    QuilcError,
    QcsSdkError,
    "Errors encountered compiling a Quil program."
);

exception!(
    crate::compiler::rpcq::Error,
    qcs_sdk.client,
    RPCQQuilcError,
    ClientError,
    "Errors when compiling with RPCQ client."
);

#[cfg(feature = "libquil")]
exception!(
    crate::compiler::libquil::Error,
    qcs_sdk.client,
    LibquilQuilcError,
    QcsSdkError,
    "Errors when compiling with the libquil client."
);

exception!(
    crate::qpu::ListQuantumProcessorsError,
    qcs_sdk,
    ListQuantumProcessorsError,
    QcsSdkError,
    "API Errors encountered when trying to list available quantum processors."
);

#[cfg(feature = "experimental")]
exception!(
    crate::qpu::experimental::random::Error,
    qcs_sdk.qpu.experimental.random,
    RandomError,
    QcsSdkError,
    "Errors that may occur using the randomization primitives defined in this module."
);

exception!(
    crate::qpu::api::QpuApiError,
    qcs_sdk.qpu.api,
    QpuApiError,
    QcsSdkError,
    "Errors that can occur while attempting to establish a connection to the QPU."
);

exception!(
    crate::qpu::api::python::SubmissionError,
    qcs_sdk.qpu.api,
    SubmissionError,
    QpuApiError,
    "Errors that may occur when submitting a program for execution."
);

exception!(
    crate::qpu::api::python::BuildOptionsError,
    qcs_sdk.qpu.api,
    BuildOptionsError,
    QpuApiError,
    "Errors building execution options."
);

exception!(
    crate::qpu::GetIsaError,
    qcs_sdk.qpu.isa,
    GetISAError,
    QcsSdkError,
    "Errors raised due to failure to get an ISA."
);

exception!(
    crate::qpu::isa::python::SerializeIsaError,
    qcs_sdk.qpu.isa,
    SerializeISAError,
    QcsSdkError,
    "Errors raised due to failure to serialize an ISA."
);

exception!(
    crate::qpu::translation::python::TranslationError,
    qcs_sdk.qpu.translation,
    TranslationError,
    QcsSdkError,
    "Errors raised due to failure to translate a program."
);

exception!(
    crate::qvm::Error,
    qcs_sdk.qvm,
    QvmError,
    QcsSdkError,
    "Errors that can occur when running a Quil program on QVM."
);
