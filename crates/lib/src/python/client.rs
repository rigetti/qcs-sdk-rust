use qcs_api_client_common::configuration::{
    secrets::SecretRefreshToken,
    settings::AuthServer,
    tokens::{ClientCredentials, ExternallyManaged, OAuthSession, RefreshToken},
    ClientConfigurationBuilder,
};

use pyo3::prelude::*;
use rigetti_pyo3::{create_init_submodule, py_sync, sync::Awaitable};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::gen_stub_pymethods;
use tokio_util::sync::CancellationToken;

use crate::client::Qcs;
use crate::python::errors;

#[cfg(not(feature = "libquil"))]
create_init_submodule! {
    classes: [
        Qcs,
        OAuthSession,
        AuthServer,
        RefreshToken,
        SecretRefreshToken,
        ClientCredentials,
        ExternallyManaged
    ],
    errors: [
        errors::BuildClientError,
        errors::ClientError,
        errors::RPCQQuilcError,
        errors::TokenError,
        errors::LoadClientError
    ],
}

// TODO (rigetti-pyo3#63): The `create_init_submodule` macro doesn't support feature-gated items.
// Developer note: putting this block second lets the `pyo3_linter` see the additional items.

#[cfg(feature = "libquil")]
create_init_submodule! {
    classes: [
        Qcs,
        OAuthSession,
        AuthServer,
        RefreshToken,
        SecretRefreshToken,
        ClientCredentials,
        ExternallyManaged
    ],
    errors: [
        errors::BuildClientError,
        errors::ClientError,
        errors::LoadClientError,
        errors::RPCQQuilcError,
        errors::TokenError,
        errors::LibquilQuilcError
    ],
}

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
        oauth_session: Option<OAuthSession>,
        api_url: Option<String>,
        grpc_api_url: Option<String>,
        quilc_url: Option<String>,
        qvm_url: Option<String>,
    ) -> PyResult<Self> {
        let mut builder = ClientConfigurationBuilder::default();

        if oauth_session.is_some() {
            builder.oauth_session(oauth_session);
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
    /// :param `profile_name`: The QCS setting's profile name to use. If ``None``, the default value configured in your environment is used.
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

    /// Create a `QCSClient` configuration using an environment-based configuration.
    ///
    /// If credentials are not found or stale, a PKCE login redirect flow will be initialized.
    /// Note that this opens up a TCP port on your system to accept a browser HTTP redirect,
    /// so you should not use this in environments where that is not possible,
    /// such as hosted `JupyterLab` sessions.
    ///
    /// :param `profile_name`: The QCS setting's profile name to use. If ``None``, the default value configured in your environment is used.
    ///
    /// :raises `LoadClientError`: If there is an issue loading the profile details from the environment or if the PKCE login flow fails.
    ///
    /// See the [QCS documentation](https://docs.rigetti.com/qcs/references/qcs-client-configuration#environment-variables-and-configuration-files)
    /// for more details.
    #[staticmethod]
    #[pyo3(signature = (/, profile_name = None))]
    fn load_with_login(py: Python<'_>, profile_name: Option<String>) -> PyResult<Self> {
        do_until_ctrl_c(move |cancel_token| {
            py_sync!(py, async move {
                Qcs::with_login(cancel_token, profile_name)
                    .await
                    .map_err(errors::ClientError::load_error)
            })
        })
    }

    /// Create a `QCSClient` configuration using an environment-based configuration.
    ///
    /// If credentials are not found or stale, a PKCE login redirect flow will be initialized.
    /// Note that this opens up a TCP port on your system to accept a browser HTTP redirect,
    /// so you should not use this in environments where that is not possible,
    /// such as hosted `JupyterLab` sessions.
    ///
    /// :param `profile_name`: The QCS setting's profile name to use. If ``None``, the default value configured in your environment is used.
    ///
    /// :raises `LoadClientError`: If there is an issue loading the profile details from the environment or if the PKCE login flow fails.
    ///
    /// See the [QCS documentation](https://docs.rigetti.com/qcs/references/qcs-client-configuration#environment-variables-and-configuration-files)
    /// for more details.
    #[staticmethod]
    #[pyo3(signature = (/, profile_name = None))]
    fn load_with_login_async(
        py: Python<'_>,
        profile_name: Option<String>,
    ) -> PyResult<Awaitable<'_, Self>> {
        do_until_ctrl_c(move |cancel_token| {
            pyo3_async_runtimes::tokio::future_into_py(py, async move {
                Qcs::with_login(cancel_token, profile_name)
                    .await
                    .map_err(errors::ClientError::load_error)
            })
            .map(Into::into)
        })
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

    /// Get a copy of the OAuth session.
    #[getter]
    fn oauth_session(&self, py: Python<'_>) -> PyResult<OAuthSession> {
        let config = self.get_config().clone();
        py_sync!(py, async move {
            config
                .oauth_session()
                .await
                .map_err(errors::ClientError::token_error)
        })
    }

    /// Get a copy of the OAuth session in an async context.
    fn get_oauth_session_async<'py>(&self, py: Python<'py>) -> PyResult<Awaitable<'py, PyAny>> {
        let config = self.get_config().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            config
                .oauth_session()
                .await
                .map_err(errors::ClientError::token_error)
        })
        .map(Into::into)
    }
}

impl PartialEq for Qcs {
    fn eq(&self, other: &Self) -> bool {
        format!("{self:?}") == format!("{other:?}")
    }
}

/// Run the given function with a [`CancellationToken`] that is cancelled when `Ctrl+C` is pressed.
fn do_until_ctrl_c<T>(f: impl FnOnce(CancellationToken) -> T) -> T {
    let cancel_token = CancellationToken::new();
    let cancel_token_ctrl_c = cancel_token.clone();
    tokio::spawn(cancel_token.clone().run_until_cancelled_owned(async move {
        drop(tokio::signal::ctrl_c().await);
        cancel_token_ctrl_c.cancel();
    }));

    let value = f(cancel_token.clone());
    cancel_token.cancel();
    value
}
