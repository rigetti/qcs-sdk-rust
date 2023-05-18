// use std::{collections::HashMap, vec::IntoIter};
use std::collections::HashMap;

// use async_zmq::{dealer, request, Message, Request, SinkExt};
use async_zmq::{dealer, Message, SinkExt};
use futures::StreamExt;
use rmp_serde::Serializer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) const DEFAULT_CLIENT_TIMEOUT: f64 = 30.0;

// Send and receive serializable messages over a ZMQ DEALER socket asynchronously.
pub(crate) async fn send_recv<Request: Serialize, Response: DeserializeOwned>(
    endpoint: &str,
    request: &RPCRequest<'_, Request>,
) -> Result<Response, Error> {
    let mut context = dealer(endpoint)
        .map_err(Error::SocketCreation)?
        .connect()
        .map_err(Error::Communication)?;

    let mut data = vec![];
    request
        .serialize(&mut Serializer::new(&mut data).with_struct_map())
        .map_err(Error::Serialization)?;

    context
        .send(Message::from(data).into())
        .await
        .map_err(Error::Send)?;
    let resp = context
        .next()
        .await
        .ok_or(Error::Empty)?
        .map_err(Error::Recv)?
        .pop()
        .ok_or(Error::Empty)?
        .to_vec();

    let reply: RPCResponse<Response> =
        rmp_serde::from_read(resp.as_slice()).map_err(Error::Deserialization)?;
    match reply {
        RPCResponse::RPCReply { id, result } => {
            if id == request.id {
                Ok(result)
            } else {
                Err(Error::ResponseIdMismatch)
            }
        }
        RPCResponse::RPCError { error, .. } => Err(Error::Response(error)),
    }
}

// /// A minimal RPCQ client that does just enough to talk to `quilc`
// pub(crate) struct Client {
//     context: Request<IntoIter<Message>, Message>,
// }

// impl Client {
//     /// Construct a new [] with no authentication configured.
//     pub(crate) fn new(endpoint: &str) -> Result<Self, Error> {
//         let context = request(endpoint)
//             .map_err(Error::SocketCreation)?
//             .connect()
//             .map_err(Error::Communication)?;
//         Ok(Self { context })
//     }

//     /// Send an RPC request and immediately retrieve and decode the results.
//     ///
//     /// # Arguments
//     ///
//     /// * `request`: An [`RPCRequest`] containing some params.
//     pub(crate) async fn run_request<Request: Serialize, Response: DeserializeOwned>(
//         &mut self,
//         request: &RPCRequest<'_, Request>,
//     ) -> Result<Response, Error> {
//         self.send(request).await?;
//         self.receive::<Response>(&request.id).await
//     }

//     /// Send an RPC request.
//     ///
//     /// # Arguments
//     ///
//     /// * `request`: An [`RPCRequest`] containing some params.
//     async fn send<Request: Serialize>(
//         &mut self,
//         request: &RPCRequest<'_, Request>,
//     ) -> Result<(), Error> {
//         let mut data = vec![];
//         request
//             .serialize(&mut Serializer::new(&mut data).with_struct_map())
//             .map_err(Error::Serialization)?;

//         // let message = MultipartIter::from(Message::from(data));
//         self.context
//             .send(Message::from(data))
//             .await
//             .map_err(Error::RequestReply)
//     }

//     /// Retrieve and decode a response
//     ///
//     /// returns: Result<Response, Error> where Response is a generic type that implements
//     /// [`DeserializeOwned`] (meaning [`Deserialize`] with no lifetimes).
//     async fn receive<Response: DeserializeOwned>(
//         &self,
//         request_id: &str,
//     ) -> Result<Response, Error> {
//         let data = self.receive_raw().await?;

//         let reply: RPCResponse<Response> =
//             rmp_serde::from_read(data.as_slice()).map_err(Error::Deserialization)?;
//         match reply {
//             RPCResponse::RPCReply { id, result } => {
//                 if id == request_id {
//                     Ok(result)
//                 } else {
//                     Err(Error::ResponseIdMismatch)
//                 }
//             }
//             RPCResponse::RPCError { error, .. } => Err(Error::Response(error)),
//         }
//     }

//     /// Retrieve the raw bytes of a response
//     async fn receive_raw(&self) -> Result<Vec<u8>, Error> {
//         let response = self.context.recv().await.map_err(Error::RequestReply)?;
//         response
//             .first()
//             .ok_or_else(|| Error::Response("Empty response".to_string()))
//             .map(|m| m.to_vec())
//     }
// }

/// All of the possible errors for this module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not create a socket.")]
    SocketCreation(#[source] async_zmq::SocketError),
    #[error("Trouble communicating with the ZMQ server")]
    Communication(#[source] async_zmq::Error),
    #[error("Trouble sending request to ZMQ server.")]
    Send(#[source] async_zmq::SendError),
    #[error("Trouble receiving response from ZMQ server.")]
    Recv(#[source] async_zmq::RecvError),
    #[error("Empty response from ZMQ server.")]
    Empty,
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
