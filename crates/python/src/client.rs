use pyo3::{exceptions::PyValueError, pyfunction};
use qcs_api_client_common::configuration::{
    AuthServer, ClientConfigurationBuilder, ClientConfigurationBuilderError, ClientCredentials,
    ExternallyManaged, OAuthSession, RefreshToken,
};
use rigetti_pyo3::{
    create_init_submodule, py_function_sync_async, py_wrap_error, py_wrap_type,
    pyo3::{
        conversion::IntoPy, exceptions::PyRuntimeError, pyclass::CompareOp, pymethods, PyObject,
        PyResult, Python,
    },
    wrap_error, PyWrapper, ToPythonError,
};

use qcs::client::{self, Qcs};

create_init_submodule! {
    classes: [
        PyQcsClient,
        OAuthSession,
        AuthServer,
        RefreshToken,
        ClientCredentials,
        ExternallyManaged
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

py_wrap_type! {
    PyQcsClient(Qcs) as "QCSClient";
}

impl PyQcsClient {
    pub fn get_or_create_client(client: Option<Self>) -> Qcs {
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
        oauth_session = None,
        api_url = None,
        grpc_api_url = None,
        quilc_url = None,
        qvm_url = None
    ))]
    pub fn new(
        oauth_session: Option<OAuthSession>,
        api_url: Option<String>,
        grpc_api_url: Option<String>,
        quilc_url: Option<String>,
        qvm_url: Option<String>,
    ) -> PyResult<Self> {
        let mut builder = ClientConfigurationBuilder::default();
        if let Some(session) = oauth_session {
            builder.oauth_session(Some(session));
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

    #[getter]
    pub fn oauth_session(&self, py: Python<'_>) -> PyResult<OAuthSession> {
        py_get_oauth_session(py, self.clone())
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyObject {
        match op {
            CompareOp::Eq => (self == other).into_py(py),
            CompareOp::Ne => (self != other).into_py(py),
            _ => py.NotImplemented(),
        }
    }
}

py_function_sync_async! {
    #[pyfunction]
    async fn get_oauth_session(client: PyQcsClient) -> PyResult<OAuthSession> {
        client.as_inner().get_config().oauth_session().await.map_err(|e| PyValueError::new_err(e.to_string()))

    }
}
