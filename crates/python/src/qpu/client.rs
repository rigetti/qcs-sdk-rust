use pyo3::{
    conversion::IntoPy, exceptions::PyRuntimeError, pymethods, types::PyDict, Py, PyAny, PyErr,
    PyResult, Python,
};
use pyo3_asyncio::tokio::future_into_py;
use qcs::qpu::Qcs;
use qcs_api_client_common::{
    configuration::{AuthServer, BuildError, ClientConfigurationBuilder, Tokens},
    ClientConfiguration,
};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_error, py_wrap_struct, py_wrap_type, wrap_error, ToPythonError,
};

create_init_submodule! {
    classes: [
        PyQcsClient,
        PyQcsClientAuthServer,
        PyQcsClientTokens
    ],
    errors: [
        QcsGrpcClientError,
        QcsGrpcEndpointError,
        QcsGrpcError,
        QcsLoadError
    ],
}

wrap_error! {
    LoadError(qcs::qpu::client::LoadError);
}
py_wrap_error!(client, LoadError, QcsLoadError, PyRuntimeError);

wrap_error! {
    GrpcError(qcs::qpu::client::GrpcError);
}
py_wrap_error!(client, GrpcError, QcsGrpcError, PyRuntimeError);

wrap_error! {
    GrpcClientError(qcs::qpu::client::GrpcClientError);
}
py_wrap_error!(client, GrpcClientError, QcsGrpcClientError, PyRuntimeError);

wrap_error! {
    GrpcEndpointError(qcs::qpu::client::GrpcEndpointError);
}
py_wrap_error!(
    client,
    GrpcEndpointError,
    QcsGrpcEndpointError,
    PyRuntimeError
);

wrap_error!(ConfigurationBuildError(BuildError));
py_wrap_error!(
    qcs,
    ConfigurationBuildError,
    QcsConfigurationBuildError,
    PyRuntimeError
);

fn get_pydict_str(py_dict: &PyDict, key: &str) -> Option<String> {
    py_dict.get_item(key)?.extract().ok()
}

py_wrap_struct! {
    PyQcsClientAuthServer(AuthServer) as "QcsClientAuthServer" {
        py -> rs {
            py_dict: Py<PyDict> => AuthServer {
                let py_dict = py_dict.as_ref(py);
                let mut auth_server = AuthServer::default();
                if let Some(client_id) = get_pydict_str(py_dict, "client_id") {
                    auth_server = auth_server.set_client_id(client_id);
                }
                if let Some(issuer) = get_pydict_str(py_dict, "issuer") {
                    auth_server = auth_server.set_issuer(issuer);
                }
                Ok::<_, PyErr>(auth_server)
            }
        },
        rs -> py {
            rs_struct: AuthServer => Py<PyDict> {
                let obj = PyDict::new(py);
                obj.set_item("client_id", rs_struct.client_id())?;
                obj.set_item("issuer", rs_struct.issuer())?;
                Ok(obj.into_py(py))
            }
        }
    }
}

py_wrap_struct! {
    PyQcsClientTokens(Tokens) as "QcsClientTokens" {
        py -> rs {
            py_dict: Py<PyDict> => Tokens {
                let py_dict = py_dict.as_ref(py);
                let bearer_access_token = get_pydict_str(py_dict, "bearer_access_token");
                let refresh_token = get_pydict_str(py_dict, "refresh_token");
                Ok::<_, PyErr>(Tokens { bearer_access_token, refresh_token })
            }
        },
        rs -> py {
            rs_struct: Tokens => Py<PyDict> {
                let obj = PyDict::new(py);
                obj.set_item("bearer_access_token", rs_struct.bearer_access_token)?;
                obj.set_item("refresh_token", rs_struct.refresh_token)?;
                Ok(obj.into_py(py))
            }
        }
    }
}

py_wrap_type! {
    PyQcsClient(Qcs) as "QcsClient";
}

impl PyQcsClient {
    pub(crate) async fn get_or_create_client(client: Option<Self>) -> PyResult<Qcs> {
        Ok(match client {
            Some(client) => client.0,
            None => Qcs::load()
                .await
                .map_err(LoadError::from)
                .map_err(LoadError::to_py_err)?,
        })
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
            builder = builder.set_tokens(tokens.0);
        }
        if let Some(api_url) = api_url {
            builder = builder.set_api_url(api_url);
        }
        if let Some(auth_server) = auth_server {
            builder = builder.set_auth_server(auth_server.0);
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
            .map_err(ConfigurationBuildError::from)
            .map_err(ConfigurationBuildError::to_py_err)?;

        Ok(Self(client))
    }

    #[staticmethod]
    #[args("/", profile_name = "None", use_gateway = "None")]
    pub fn load(
        py: Python<'_>,
        profile_name: Option<String>,
        use_gateway: Option<bool>,
    ) -> PyResult<&PyAny> {
        future_into_py(py, async move {
            let config = match profile_name {
                Some(profile_name) => ClientConfiguration::load_profile(profile_name).await,
                None => ClientConfiguration::load_default().await,
            };

            let client = config
                .map(Qcs::with_config)
                .map_err(LoadError)
                .map_err(ToPythonError::to_py_err)?;

            let client = match use_gateway {
                None => client,
                Some(use_gateway) => client.with_use_gateway(use_gateway),
            };

            Ok(Self(client))
        })
    }

    fn info(&self) -> String {
        format!("{:?}", self.0)
    }
}
