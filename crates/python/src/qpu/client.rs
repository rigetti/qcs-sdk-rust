use qcs_api_client_common::{
    configuration::{AuthServer, BuildError, ClientConfigurationBuilder, Tokens},
    ClientConfiguration,
};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct, py_wrap_error, py_wrap_type,
    pyo3::{
        conversion::IntoPy, exceptions::PyRuntimeError, pyclass, pyclass::CompareOp, pymethods,
        types::PyString, FromPyObject, Py, PyAny, PyObject, PyResult, Python,
    },
    wrap_error, ToPythonError,
};

use qcs::qpu::{client, Qcs};

use crate::py_sync::{py_async, py_sync};

create_init_submodule! {
    classes: [
        PyQcsClient,
        PyQcsClientAuthServer,
        PyQcsClientTokens
    ],
    errors: [
        // ClientGrpcError,
        // GrpcEndpointError,
        LoadClientError,
        BuildClientError
    ],
}

wrap_error!(RustLoadClientError(client::LoadError));
py_wrap_error!(client, RustLoadClientError, LoadClientError, PyRuntimeError);

wrap_error!(RustBuildClientError(BuildError));
py_wrap_error!(
    client,
    RustBuildClientError,
    BuildClientError,
    PyRuntimeError
);

// wrap_error!(RustClientGrpcError(client::GrpcError));
// py_wrap_error!(client, RustClientGrpcError, ClientGrpcError, PyRuntimeError);

// wrap_error!(RustGrpcEndpointError(client::GrpcEndpointError));
// py_wrap_error!(
//     client,
//     RustGrpcEndpointError,
//     GrpcEndpointError,
//     PyRuntimeError
// );

/// The fields on qcs_api_client_common::client::AuthServer are not public.
#[pyclass]
#[pyo3(name = "QCSClientAuthServer")]
#[derive(FromPyObject)]
pub struct PyQcsClientAuthServer {
    #[pyo3(get, set)]
    pub client_id: Option<String>,
    #[pyo3(get, set)]
    pub issuer: Option<String>,
}

impl From<PyQcsClientAuthServer> for AuthServer {
    fn from(value: PyQcsClientAuthServer) -> Self {
        let mut auth_server = AuthServer::default();
        if let Some(client_id) = value.client_id {
            auth_server = auth_server.set_client_id(client_id);
        }
        if let Some(issuer) = value.issuer {
            auth_server = auth_server.set_issuer(issuer);
        }
        auth_server
    }
}

#[pymethods]
impl PyQcsClientAuthServer {
    #[new]
    #[args(client_id = "None", issuer = "None")]
    pub fn new(client_id: Option<String>, issuer: Option<String>) -> Self {
        Self { client_id, issuer }
    }
}

py_wrap_data_struct! {
    PyQcsClientTokens(Tokens) as "QCSClientTokens" {
        bearer_access_token: Option<String> => Option<Py<PyString>>,
        refresh_token: Option<String> => Option<Py<PyString>>
    }
}

#[pymethods]
impl PyQcsClientTokens {
    #[new]
    #[args(bearer_access_token = "None", refresh_token = "None")]
    pub fn new(bearer_access_token: Option<String>, refresh_token: Option<String>) -> Self {
        Self(Tokens {
            bearer_access_token,
            refresh_token,
        })
    }
}

py_wrap_type! {
    PyQcsClient(Qcs) as "QCSClient";
}

impl PyQcsClient {
    pub(crate) async fn get_or_create_client(client: Option<Self>) -> PyResult<Qcs> {
        Ok(match client {
            Some(client) => client.into(),
            None => Qcs::load()
                .await
                .map_err(RustLoadClientError::from)
                .map_err(RustLoadClientError::to_py_err)?,
        })
    }

    async fn load(profile_name: Option<String>, use_gateway: Option<bool>) -> PyResult<Self> {
        let config = match profile_name {
            Some(profile_name) => ClientConfiguration::load_profile(profile_name).await,
            None => ClientConfiguration::load_default().await,
        };

        let client = config
            .map(Qcs::with_config)
            .map_err(RustLoadClientError)
            .map_err(RustLoadClientError::to_py_err)?;

        let client = match use_gateway {
            None => client,
            Some(use_gateway) => client.with_use_gateway(use_gateway),
        };

        Ok(Self(client))
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
    #[args(
        "/",
        tokens = "None",
        api_url = "None",
        auth_server = "None",
        grpc_api_url = "None",
        quilc_url = "None",
        qvm_url = "None"
    )]
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
            builder = builder.set_tokens(tokens.into());
        }
        if let Some(api_url) = api_url {
            builder = builder.set_api_url(api_url);
        }
        if let Some(auth_server) = auth_server {
            builder = builder.set_auth_server(auth_server.into());
        }
        if let Some(grpc_api_url) = grpc_api_url {
            builder = builder.set_grpc_api_url(grpc_api_url);
        }
        if let Some(quilc_url) = quilc_url {
            builder = builder.set_quilc_url(quilc_url);
        }
        if let Some(qvm_url) = qvm_url {
            builder = builder.set_grpc_api_url(qvm_url);
        }
        let client = builder
            .build()
            .map(Qcs::with_config)
            .map_err(RustBuildClientError::from)
            .map_err(RustBuildClientError::to_py_err)?;

        Ok(Self(client))
    }

    #[staticmethod]
    #[args("/", profile_name = "None", use_gateway = "None")]
    #[pyo3(name = "load")]
    pub fn py_load(profile_name: Option<String>, use_gateway: Option<bool>) -> PyResult<Self> {
        py_sync!(Self::load(profile_name, use_gateway))
    }

    #[staticmethod]
    #[args("/", profile_name = "None", use_gateway = "None")]
    #[pyo3(name = "load_async")]
    pub fn py_load_async(
        py: Python<'_>,
        profile_name: Option<String>,
        use_gateway: Option<bool>,
    ) -> PyResult<&PyAny> {
        py_async!(py, Self::load(profile_name, use_gateway))
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
