use std::collections::HashMap;

use rmp_serde::Serializer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zmq::{Context, Socket, SocketType};

pub(crate) const DEFAULT_CLIENT_TIMEOUT: f64 = 30.0;

/// A minimal RPCQ client that does just enough to talk to `quilc`
pub(crate) struct Client {
    socket: Socket,
}

impl Client {
    /// Construct a new [`Client`] with no authentication configured.
    pub(crate) fn new(endpoint: &str) -> Result<Self, Error> {
        let socket = Context::new()
            .socket(SocketType::DEALER)
            .map_err(Error::SocketCreation)?;
        socket.connect(endpoint).map_err(Error::Communication)?;
        Ok(Self { socket })
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
        self.send(request)?;
        self.receive::<Response>(&request.id)
    }

    /// Send an RPC request.
    ///
    /// # Arguments
    ///
    /// * `request`: An [`RPCRequest`] containing some params.
    pub(crate) fn send<Request: Serialize>(
        &self,
        request: &RPCRequest<'_, Request>,
    ) -> Result<(), Error> {
        let mut data = vec![];
        request
            .serialize(&mut Serializer::new(&mut data).with_struct_map())
            .map_err(Error::Serialization)?;

        self.socket.send(data, 0).map_err(Error::Communication)
    }

    /// Retrieve and decode a response
    ///
    /// returns: Result<Response, Error> where Response is a generic type that implements
    /// [`DeserializeOwned`] (meaning [`Deserialize`] with no lifetimes).
    fn receive<Response: DeserializeOwned>(&self, request_id: &str) -> Result<Response, Error> {
        let data = self.receive_raw()?;

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
    pub(crate) fn receive_raw(&self) -> Result<Vec<u8>, Error> {
        self.socket.recv_bytes(0).map_err(Error::Communication)
    }
}

/// All of the possible errors for this module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not create a socket.")]
    SocketCreation(#[source] zmq::Error),
    #[error("Failed while trying to set up auth. This is likely a bug in this library.")]
    AuthSetup(#[source] zmq::Error),
    #[error("Trouble communicating with the ZMQ server")]
    Communication(#[source] zmq::Error),
    #[error("Could not serialize request as MessagePack. This is a bug in this library.")]
    Serialization(#[from] rmp_serde::encode::Error),
    #[error("Could not decode ZMQ server's response. This is likely a bug in this library.")]
    Deserialization(#[from] rmp_serde::decode::Error),
    #[error("Response ID did not match request ID")]
    ResponseIdMismatch,
    #[error("Received error message from server: {0}")]
    Response(String),
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
