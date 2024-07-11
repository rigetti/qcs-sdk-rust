use qcs_api_client_common::configuration::{
    AuthServer, ClientConfigurationBuilder, ClientConfigurationBuilderError, Tokens,
};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct, py_wrap_error, py_wrap_type,
    pyo3::{
        conversion::IntoPy, exceptions::PyRuntimeError, pyclass::CompareOp, pymethods,
        types::PyString, Py, PyObject, PyResult, Python,
    },
    wrap_error, ToPythonError,
};

use qcs::client::{self, Qcs};

create_init_submodule! {
    classes: [
        PyQcsClient,
        PyQcsClientAuthServer,
        PyQcsClientTokens
    ],
    errors: [
        LoadClientError,
        BuildClientError
    ],
}

wrap_error!(RustLoadClientError(client::LoadError));
py_wrap_error!(client, RustLoadClientError, LoadClientError, PyRuntimeError);

wrap_error!(RustBuildClientError(ClientConfigurationBuilderError));

py_wrap_error!(
    client,
    RustBuildClientError,
    BuildClientError,
    PyRuntimeError
);

// The fields on qcs_api_client_common::client::AuthServer are not public.
py_wrap_type!(
    PyQcsClientAuthServer(AuthServer) as "QCSClientAuthServer"
);

#[pymethods]
impl PyQcsClientAuthServer {
    #[new]
    #[pyo3(signature = (client_id = None, issuer = None))]
    pub fn new(client_id: Option<String>, issuer: Option<String>) -> Self {
        let mut auth_server = AuthServer::default();
        if let Some(client_id) = client_id {
            auth_server.set_client_id(client_id);
        }
        if let Some(issuer) = issuer {
            auth_server.set_issuer(issuer);
        }
        Self(auth_server)
    }

    #[getter(client_id)]
    fn get_client_id(&self) -> String {
        self.0.client_id().to_string()
    }

    #[setter(client_id)]
    fn set_client_id(&mut self, value: String) {
        self.0.set_client_id(value);
    }

    #[getter(issuer)]
    fn get_issuer(&self) -> String {
        self.0.issuer().to_string()
    }

    #[setter(issuer)]
    fn set_issuer(&mut self, value: String) {
        self.0.set_issuer(value);
    }
}

py_wrap_data_struct! {
    PyQcsClientTokens(Tokens) as "QCSClientTokens" {
        bearer_access_token: String => Py<PyString>,
        refresh_token: String => Py<PyString>,
        auth_server: AuthServer => PyQcsClientAuthServer
    }
}

#[pymethods]
impl PyQcsClientTokens {
    #[new]
    #[pyo3(signature = (bearer_access_token, refresh_token, auth_server = None))]
    pub fn new(
        bearer_access_token: String,
        refresh_token: String,
        auth_server: Option<PyQcsClientAuthServer>,
    ) -> Self {
        Self(Tokens {
            bearer_access_token,
            refresh_token,
            auth_server: auth_server.map(Into::into).unwrap_or_default(),
        })
    }
}

py_wrap_type! {
    PyQcsClient(Qcs) as "QCSClient";
}

impl PyQcsClient {
    pub(crate) fn get_or_create_client(client: Option<Self>) -> Qcs {
        match client {
            Some(client) => client.into(),
            None => Qcs::load(),
        }
    }
}

impl PartialEq for PyQcsClient {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self.0) == format!("{:?}", other.0)
    }
}

#[pymethods]
impl PyQcsClient {
    #[new]
    #[pyo3(signature = (
        /,
        tokens = None,
        api_url = None,
        auth_server = None,
        grpc_api_url = None,
        quilc_url = None,
        qvm_url = None
    ))]
    pub fn new(
        tokens: Option<PyQcsClientTokens>,
        api_url: Option<String>,
        auth_server: Option<PyQcsClientAuthServer>,
        grpc_api_url: Option<String>,
        quilc_url: Option<String>,
        qvm_url: Option<String>,
    ) -> PyResult<Self> {
        let mut builder = ClientConfigurationBuilder::default();
        if let Some(tokens) = tokens {
            builder.tokens(Some(tokens.into()));
        }
        if let Some(api_url) = api_url {
            builder.api_url(api_url);
        }
        if let Some(auth_server) = auth_server {
            builder.auth_server(auth_server.into());
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
            .map_err(RustBuildClientError::from)
            .map_err(RustBuildClientError::to_py_err)?;

        Ok(Self(client))
    }

    #[staticmethod]
    #[pyo3(signature = (/, profile_name = None))]
    fn load(profile_name: Option<String>) -> PyResult<Self> {
        Ok(match profile_name {
            Some(profile_name) => Qcs::with_profile(profile_name)
                .map(PyQcsClient)
                .map_err(RustLoadClientError)
                .map_err(RustLoadClientError::to_py_err)?,
            None => Self(Qcs::load()),
        })
    }

    #[getter]
    pub fn api_url(&self) -> String {
        self.as_ref().get_config().api_url().to_string()
    }

    #[getter]
    pub fn grpc_api_url(&self) -> String {
        self.as_ref().get_config().grpc_api_url().to_string()
    }

    #[getter]
    pub fn quilc_url(&self) -> String {
        self.as_ref().get_config().quilc_url().to_string()
    }

    #[getter]
    pub fn qvm_url(&self) -> String {
        self.as_ref().get_config().qvm_url().to_string()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
        match op {
            CompareOp::Eq => (self == other).into_py(py),
            CompareOp::Ne => (self != other).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}
