//! Provides an RPCQ client for Quilc

use std::collections::HashMap;
use std::str::FromStr;

use quil_rs::Program;
use rmp_serde::Serializer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

use super::quilc;

pub(crate) const DEFAULT_CLIENT_TIMEOUT: f64 = 30.0;

/// A minimal RPCQ client that does just enough to talk to `quilc`
#[derive(Clone)]
pub struct Client {
    pub(crate) endpoint: String,
    send_timeout: Option<i32>,
    receive_timeout: Option<i32>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPCQ client for {}", self.endpoint)
    }
}

impl Client {
    /// Construct a new [`Client`] with no authentication configured.
    pub fn new(endpoint: &str) -> Result<Self, Error> {
        Ok(Self {
            endpoint: endpoint.to_owned(),
            send_timeout: None,
            receive_timeout: None,
        })
    }

    /// Set the timeout used for both sending and receiving messages
    ///
    /// Value is number of milliseconds. A value of `-1` means no timeout.
    pub fn set_timeout(&mut self, timeout: i32) {
        self.set_send_timeout(timeout);
        self.set_receive_timeout(timeout);
    }

    /// Set the timeout used when sending messages
    ///
    /// Value is number of milliseconds. A value of `-1` means no timeout.
    pub fn set_send_timeout(&mut self, timeout: i32) {
        self.send_timeout = Some(timeout);
    }

    /// Set the timeout used when receiving messages
    ///
    /// Value is number of milliseconds. A value of `-1` means no timeout.
    pub fn set_receive_timeout(&mut self, timeout: i32) {
        self.receive_timeout = Some(timeout);
    }

    /// Send an RPC request and immediately retrieve and decode the results.
    ///
    /// # Arguments
    ///
    /// * `request`: An [`RPCRequest`] containing some params.
    pub(crate) fn run_request<Request: Serialize, Response: DeserializeOwned>(
        &self,
        request: &RPCRequest<'_, Request>,
    ) -> Result<Response, Error> {
        let socket = self.create_socket()?;
        Self::send(request, &socket)?;
        Self::receive::<Response>(&request.id, &socket)
    }

    /// Send an RPC request.
    ///
    /// # Arguments
    ///
    /// * `request`: An [`RPCRequest`] containing some params.
    /// * `socket`: The ZMQ socket to send the request on.
    fn send<Request: Serialize>(
        request: &RPCRequest<'_, Request>,
        socket: &Socket,
    ) -> Result<(), Error> {
        let mut data = vec![];
        request
            .serialize(&mut Serializer::new(&mut data).with_struct_map())
            .map_err(Error::Serialization)?;

        socket.send(data, 0).map_err(Error::Communication)
    }

    /// Creates a new ZMQ socket and connects it to the endpoint.
    ///
    /// [`SocketType::DEALER`] for compatiblity with the quilc servers
    /// [`SocketType::ROUTER`]. These sockets are _not_ thread safe, even
    /// with a mutex, so a new socket should be created for each request,
    /// and the socket should not be shared between threads.
    ///
    /// If [`Self::set_send_timeout`] and/or [`Self::set_receive_timeout`]
    /// have been used to set a timeout, it will be applied here to the
    /// returned [`Socket`].
    fn create_socket(&self) -> Result<Socket, Error> {
        let socket = Context::new()
            .socket(SocketType::DEALER)
            .map_err(Error::SocketCreation)?;
        if let Some(send_timeout) = self.send_timeout {
            socket
                .set_sndtimeo(send_timeout)
                .map_err(Error::Communication)?;
        }
        if let Some(receive_timeout) = self.receive_timeout {
            socket
                .set_rcvtimeo(receive_timeout)
                .map_err(Error::Communication)?;
        }
        socket
            .connect(&self.endpoint.clone())
            .map_err(Error::Communication)?;
        socket.set_linger(0).map_err(Error::Communication)?;
        Ok(socket)
    }

    /// Retrieve and decode a response
    ///
    /// returns: Result<Response, Error> where Response is a generic type that implements
    /// [`DeserializeOwned`] (meaning [`Deserialize`] with no lifetimes).
    fn receive<Response: DeserializeOwned>(
        request_id: &str,
        socket: &Socket,
    ) -> Result<Response, Error> {
        let data = Self::receive_raw(socket)?;

        let reply: RPCResponse<Response> =
            rmp_serde::from_read(data.as_slice()).map_err(Error::Deserialization)?;
        match reply {
            RPCResponse::RPCReply { id, result } => {
                if id == request_id {
                    Ok(result)
                } else {
                    Err(Error::ResponseIdMismatch)
                }
            }
            RPCResponse::RPCError { error, .. } => Err(Error::Response(error)),
        }
    }

    /// Retrieve the raw bytes of a response
    fn receive_raw(socket: &Socket) -> Result<Vec<u8>, Error> {
        socket.recv_bytes(0).map_err(Error::Communication)
    }
}

impl quilc::Client for Client {
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    fn compile_program(
        &self,
        quil: &str,
        isa: quilc::TargetDevice,
        options: quilc::CompilerOpts,
    ) -> Result<quilc::CompilationResult, quilc::Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(compiler_options=?options, "compiling quil program with quilc (RPCQ)",);
        let params = quilc::QuilcParams::new(quil, isa).with_protoquil(options.protoquil);
        let request = RPCRequest::new("quil_to_native_quil", &params).with_timeout(options.timeout);
        match self.run_request::<_, quilc::QuilToNativeQuilResponse>(&request) {
            Ok(response) => Ok(quilc::CompilationResult {
                program: Program::from_str(&response.quil).map_err(quilc::Error::Parse)?,
                native_quil_metadata: response.metadata,
            }),
            Err(source) => Err(Error::to_quilc_error(self.endpoint.clone(), source)),
        }
    }

    fn get_version_info(&self) -> Result<String, quilc::Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("requesting quilc version information");

        // todo check this hashmap type
        let bindings: HashMap<String, String> = HashMap::new();
        let request = RPCRequest::new("get_version_info", &bindings);
        match self.run_request::<_, quilc::QuilcVersionResponse>(&request) {
            Ok(response) => Ok(response.quilc),
            Err(source) => Err(Error::to_quilc_error(self.endpoint.clone(), source)),
        }
    }

    fn conjugate_pauli_by_clifford(
        &self,
        request: quilc::ConjugateByCliffordRequest,
    ) -> Result<quilc::ConjugatePauliByCliffordResponse, quilc::Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("requesting quilc conjugate_pauli_by_clifford");

        let request: quilc::ConjugatePauliByCliffordRequest = request.into();
        let request = RPCRequest::new("conjugate_pauli_by_clifford", &request);
        match self.run_request::<_, quilc::ConjugatePauliByCliffordResponse>(&request) {
            Ok(response) => Ok(response),
            Err(source) => Err(Error::to_quilc_error(self.endpoint.clone(), source)),
        }
    }

    fn generate_randomized_benchmarking_sequence(
        &self,
        request: quilc::RandomizedBenchmarkingRequest,
    ) -> Result<quilc::GenerateRandomizedBenchmarkingSequenceResponse, quilc::Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!("requesting quilc generate_randomized_benchmarking_sequence");

        let request: quilc::GenerateRandomizedBenchmarkingSequenceRequest = request.into();
        let request = RPCRequest::new("generate_rb_sequence", &request);
        match self.run_request::<_, quilc::GenerateRandomizedBenchmarkingSequenceResponse>(&request)
        {
            Ok(response) => Ok(response),
            Err(source) => Err(Error::to_quilc_error(self.endpoint.clone(), source)),
        }
    }
}

/// All of the possible errors for this module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The ZMQ socket could not be created
    #[error("Could not create a socket: {0}")]
    SocketCreation(#[source] zmq::Error),
    /// Failed to set up auth for ZMQ
    #[error("Failed while trying to set up auth. This is likely a bug in this library.")]
    AuthSetup(#[source] zmq::Error),
    /// Encountered error when communicating with server
    #[error("Trouble communicating with the ZMQ server: {0}")]
    Communication(#[source] zmq::Error),
    /// Failed to serialize request
    #[error("Could not serialize request as MessagePack. This is a bug in this library: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),
    /// Failed to deserialize response
    #[error("Could not decode ZMQ server's response. This is likely a bug in this library: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),
    /// Response ID did not match request ID
    #[error("Response ID did not match request ID")]
    ResponseIdMismatch,
    /// Server responded with an error message
    #[error("Received error message from server: {0}")]
    Response(String),
    /// Error occurred when trying to lock the ZMQ socket
    #[error("Could not lock RPCQ client: {0}")]
    ZmqSocketLock(String),
}

impl Error {
    pub(crate) fn to_quilc_error(quilc_uri: String, source: Error) -> quilc::Error {
        match source {
            Error::Response(_) => {
                quilc::Error::QuilcCompilation(quilc::CompilationError::Rpcq(source))
            }
            source => quilc::Error::QuilcConnection(quilc_uri, source),
        }
    }
}

/// A single request object according to the JSONRPC standard.
///
/// Construct this using [`RPCRequest::new`]
#[derive(Serialize)]
#[serde(tag = "_type")]
pub(crate) struct RPCRequest<'params, T = HashMap<String, String>>
where
    T: Serialize,
{
    method: &'static str,
    params: &'params T,
    id: String,
    jsonrpc: &'static str,
    client_timeout: Option<f64>,
    client_key: Option<String>,
}

impl<'params, T: Serialize> RPCRequest<'params, T> {
    /// Construct a new [`RPCRequest`] to send via [`Client::run_request`] or [`Client::send`].
    ///
    /// # Arguments
    ///
    /// * `method`: The name of the RPC method to call on the server.
    /// * `params`: The parameters to send. This must implement [`serde::Serialize`].
    ///
    /// returns: `RPCRequest<T>` where `T` is the type you passed in as `params`.
    pub(crate) fn new(method: &'static str, params: &'params T) -> Self {
        Self {
            method,
            params,
            id: Uuid::new_v4().to_string(),
            jsonrpc: "2.0",
            client_timeout: Some(DEFAULT_CLIENT_TIMEOUT),
            client_key: None,
        }
    }

    /// Sets the client timeout for the [`RPCRequest`].
    ///
    /// # Arguments
    ///
    /// * `seconds`: The number of seconds to wait before timing out, or None for no timeout
    ///
    /// returns: `RPCRequest<T>` with the updated timeout.
    pub(crate) fn with_timeout(mut self, seconds: Option<f64>) -> Self {
        self.client_timeout = seconds;
        self
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_type")]
pub(crate) enum RPCResponse<T> {
    RPCReply { id: String, result: T },
    RPCError { error: String },
}
