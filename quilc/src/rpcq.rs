use std::collections::HashMap;

use rmp_serde::Serializer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zmq::{Context, SocketType};

pub(crate) fn send_request<Response: DeserializeOwned, Request: Serialize>(
    request: &RPCRequest<Request>,
    endpoint: &str,
) -> Result<Response, Error> {
    let mut data = vec![];
    request.serialize(&mut Serializer::new(&mut data).with_struct_map())?;

    let socket = Context::new().socket(SocketType::DEALER)?;

    socket.connect(endpoint)?;

    socket.send(data, 0)?;
    let data = socket.recv_bytes(0)?;

    let reply: RPCResponse<Response> = rmp_serde::from_read(data.as_slice())?;
    match reply {
        RPCResponse::RPCReply { id, result } => {
            if id == request.id {
                Ok(result)
            } else {
                Err(Error::IdMismatch)
            }
        }
        RPCResponse::RPCError { error, .. } => Err(Error::Server(error)),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not serialize request")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("Could not deserialize response")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("Could not communicate with quilc")]
    Communication(#[from] zmq::Error),
    #[error("Response ID did not match request ID")]
    IdMismatch,
    #[error("The RPC error replied with an error message")]
    Server(String),
}

/// A single request object according to the JSONRPC standard.
#[derive(Serialize)]
#[serde(tag = "_type")]
pub(crate) struct RPCRequest<T = HashMap<String, String>> {
    method: String,
    params: T,
    id: String,
    jsonrpc: String,
    client_timeout: u8,
    client_key: Option<String>,
}

impl<T> RPCRequest<T> {
    pub(crate) fn new(method: String, params: T) -> Self {
        Self {
            method,
            params,
            id: Uuid::new_v4().to_string(),
            jsonrpc: "2.0".to_string(),
            client_timeout: 10,
            client_key: None,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_type")]
enum RPCResponse<T> {
    RPCReply { id: String, result: T },
    RPCError { id: String, error: String },
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::rpcq::{send_request, RPCRequest};
    use std::collections::HashMap;

    #[derive(Deserialize, Debug)]
    struct VersionResult {
        quilc: String,
        githash: String,
    }

    /// This test serves as basic validation of zmq <-> quilc
    #[test]
    fn test_get_version() {
        let config = qcs_util::Configuration::default();
        let params: HashMap<String, String> = HashMap::new();
        let request = RPCRequest::new("get_version_info".to_string(), params);
        let resp: VersionResult =
            send_request(&request, &config.quilc_url).expect("Failed to talk to quilc");
        let version_parts: Vec<&str> = resp.quilc.split(".").collect();
        // We can't guarantee the quilc version, but this has only been tested with major version 1
        // so we'll just check for that.
        assert_eq!(version_parts[0], "1");
    }
}
