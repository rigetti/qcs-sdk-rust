use qcs_api_client_common::configuration::{self, ClientConfigurationBuilder};

use pyo3::prelude::*;

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use crate::client::Qcs;
use crate::py_sync;
use crate::python::{errors, py_function_sync_async};

macro_rules! py_wrap {
    (struct $name:ident ($py:ty)) => {
        #[derive(Clone)]
        #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
        #[cfg_attr(feature = "stubs", gen_stub_pyclass)]
        #[pyclass(module = "qcs_sdk.client")]
        struct $name($py);
    };
}

py_wrap!(struct OAuthSession(configuration::OAuthSession));
py_wrap!(struct AuthServer(configuration::AuthServer));
py_wrap!(struct RefreshToken(configuration::RefreshToken));
py_wrap!(struct ClientCredentials(configuration::ClientCredentials));
py_wrap!(struct ExternallyManaged(configuration::ExternallyManaged));

#[pymodule]
#[pyo3(name = "client", module = "qcs_sdk", submodule)]
pub(super) fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    m.add(
        "BuildClientError",
        py.get_type::<errors::BuildClientError>(),
    )?;
    m.add("ClientError", py.get_type::<errors::ClientError>())?;
    m.add("LoadClientError", py.get_type::<errors::LoadClientError>())?;

    m.add_class::<Qcs>()?;
    m.add_class::<OAuthSession>()?;
    m.add_class::<AuthServer>()?;
    m.add_class::<RefreshToken>()?;
    m.add_class::<ClientCredentials>()?;
    m.add_class::<ExternallyManaged>()?;

    Ok(())
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Qcs {
    /// Manually construct a `QCSClient`.
    ///
    /// Prefer to use `QCSClient.load` to construct an environment-based profile.
    #[new]
    #[pyo3(signature = (
        oauth_session = None,
        api_url = None,
        grpc_api_url = None,
        quilc_url = None,
        qvm_url = None
    ))]
    fn __new__(
        #[gen_stub(
            override_type(
                type_repr="OAuthSession",
                imports=("qcs_api_client_common.configuration")
            )
        )]
        oauth_session: Option<OAuthSession>,
        api_url: Option<String>,
        grpc_api_url: Option<String>,
        quilc_url: Option<String>,
        qvm_url: Option<String>,
    ) -> PyResult<Self> {
        let mut builder = ClientConfigurationBuilder::default();

        if let Some(oauth_session) = oauth_session {
            builder.oauth_session(Some(oauth_session.0));
        }
        if let Some(api_url) = api_url {
            builder.api_url(api_url);
        }
        if let Some(grpc_api_url) = grpc_api_url {
            builder.grpc_api_url(grpc_api_url);
        }
        if let Some(quilc_url) = quilc_url {
            builder.quilc_url(quilc_url);
        }
        if let Some(qvm_url) = qvm_url {
            builder.qvm_url(qvm_url);
        }

        let client = builder
            .build()
            .map(Qcs::with_config)
            .map_err(errors::ClientError::builder_error)?;

        Ok(client)
    }

    /// Create a `QCSClient` configuration using an environment-based configuration.
    ///
    /// :param profile_name: The QCS setting's profile name to use. If ``None``, the default value configured in your environment is used.
    ///
    /// :raises `LoadClientError`: If there is an issue loading the profile details from the environment.
    #[staticmethod]
    #[pyo3(name = "load", signature = (profile_name = None))]
    fn py_load(profile_name: Option<String>) -> PyResult<Self> {
        match profile_name {
            None => Ok(Qcs::load()),
            Some(profile_name) => {
                Qcs::with_profile(profile_name).map_err(errors::ClientError::load_error)
            }
        }
    }

    /// URL to access the QCS API.
    #[getter]
    fn api_url(&self) -> String {
        self.get_config().api_url().to_string()
    }

    /// URL to access the QCS gRPC API.
    #[getter]
    fn grpc_api_url(&self) -> String {
        self.get_config().grpc_api_url().to_string()
    }

    /// URL to access the ``quilc`` compiler.
    #[getter]
    fn quilc_url(&self) -> String {
        self.get_config().quilc_url().to_string()
    }

    /// URL to access the QVM.
    #[getter]
    fn qvm_url(&self) -> String {
        self.get_config().qvm_url().to_string()
    }

    /// Get a copy of the OAuth session in an async context.
    fn oauth_session_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let config = self.get_config().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            config
                .oauth_session()
                .await
                .map(OAuthSession)
                .map_err(errors::ClientError::token_error)
        })
    }

    /// Get a copy of the OAuth session.
    fn oauth_session<'py>(&self, py: Python<'py>) -> PyResult<OAuthSession> {
        let config = self.get_config().clone();
        py_sync!(py, async move {
            config
                .oauth_session()
                .await
                .map(OAuthSession)
                .map_err(errors::ClientError::token_error)
        })
    }
}

impl PartialEq for Qcs {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

py_function_sync_async! {
    /// Get a copy of the OAuth session.
    #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.client"))]
    #[pyfunction]
    async fn get_oauth_session(client: Qcs) -> PyResult<OAuthSession> {
        client
            .get_config()
            .oauth_session()
            .await
            .map(OAuthSession)
            .map_err(errors::ClientError::token_error)
    }
}
