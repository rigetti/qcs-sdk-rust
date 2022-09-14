//! Use some local servers to stub out real requests to QCS in order to test the end to end flow of
//! the `qcs` crate.

use std::thread;
use std::time::Duration;

use maplit::hashmap;

use qcs::configuration::{SECRETS_PATH_VAR, SETTINGS_PATH_VAR};
use qcs::{Executable, RegisterData};

const BELL_STATE: &str = r#"
DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"#;

const QPU_ID: &str = "Aspen-9";

#[tokio::test]
async fn successful_bell_state() {
    setup().await;
    let result = Executable::from_quil(BELL_STATE)
        .with_shots(2)
        .execute_on_qpu(QPU_ID)
        .await
        .expect("Failed to run program that should be successful");
    assert_eq!(
        result.registers,
        hashmap! {Box::from(String::from("ro")) => RegisterData::I8(vec![vec![0, 0], vec![1, 1]])}
    );
    assert_eq!(result.duration, Some(Duration::from_micros(8675)));
}

async fn setup() {
    simple_logger::init_with_env().unwrap();
    std::env::set_var(SETTINGS_PATH_VAR, "tests/settings.toml");
    std::env::set_var(SECRETS_PATH_VAR, "tests/secrets.toml");
    thread::spawn(qpu::run);
    tokio::spawn(auth_server::run());
    tokio::spawn(mock_qcs::run());
}

#[allow(dead_code)]
mod auth_server {
    use serde::{Deserialize, Serialize};
    use warp::Filter;

    #[derive(Debug, Deserialize)]
    struct TokenRequest {
        grant_type: String,
        client_id: String,
        refresh_token: String,
    }

    #[derive(Serialize, Debug)]
    struct TokenResponse {
        refresh_token: &'static str,
        access_token: &'static str,
    }

    pub(crate) async fn run() {
        let token = warp::post()
            .and(warp::path("v1").and(warp::path("token")))
            .and(warp::body::form())
            .map(|_request: TokenRequest| {
                warp::reply::json(&TokenResponse {
                    refresh_token: "refreshed",
                    access_token: "accessed",
                })
            });
        warp::serve(token).run(([127, 0, 0, 1], 8001)).await;
    }
}

#[allow(dead_code)]
mod mock_qcs {
    use serde::{Deserialize, Serialize};
    use warp::Filter;

    use qcs_api::models::{
        CreateEngagementRequest, EngagementCredentials, EngagementWithCredentials,
        InstructionSetArchitecture, TranslateNativeQuilToEncryptedBinaryRequest,
        TranslateNativeQuilToEncryptedBinaryResponse,
    };

    use super::QPU_ID;

    #[derive(Debug, Deserialize)]
    struct TokenRequest {
        grant_type: String,
        client_id: String,
        refresh_token: String,
    }

    #[derive(Serialize, Debug)]
    struct TokenResponse {
        refresh_token: &'static str,
        access_token: &'static str,
    }

    pub(crate) async fn run() {
        let isa = warp::path(QPU_ID)
            .and(warp::path("instructionSetArchitecture"))
            .and(warp::get())
            .map(|| {
                let isa = std::fs::read_to_string("tests/aspen_9_isa.json")
                    .expect("Could not load Aspen 9 ISA");
                let isa: InstructionSetArchitecture =
                    serde_json::from_str(&isa).expect("Could not decode aspen 9 ISA");
                warp::reply::json(&isa)
            });

        let translate = warp::path(format!("{}:translateNativeQuilToEncryptedBinary", QPU_ID))
            .and(warp::post())
            .and(warp::body::json())
            .map(|_request: TranslateNativeQuilToEncryptedBinaryRequest| {
                warp::reply::json(&TranslateNativeQuilToEncryptedBinaryResponse {
                    memory_descriptors: None,
                    program: "".to_string(),
                    ro_sources: Some(vec![
                        vec!["ro[0]".to_string(), "q0".to_string()],
                        vec!["q0_unclassified".to_string(), "q0_unclassified".to_string()],
                        vec!["ro[1]".to_string(), "q1".to_string()],
                        vec!["q1_unclassified".to_string(), "q1_unclassified".to_string()],
                    ]),
                    settings_timestamp: None,
                })
            });
        let quantum_processors = warp::path("quantumProcessors").and(isa.or(translate));

        let engagements = warp::path("engagements")
            .and(warp::post())
            .and(warp::body::json())
            .map(|_request: CreateEngagementRequest| {
                warp::reply::json(&EngagementWithCredentials {
                    account_id: None,
                    account_type: None,
                    address: "tcp://localhost:8002".to_string(),
                    credentials: Box::new(EngagementCredentials {
                        client_public: String::new(),
                        client_secret: String::new(),
                        server_public: String::new(),
                    }),
                    endpoint_id: "".to_string(),
                    expires_at: "".to_string(),
                    minimum_priority: None,
                    quantum_processor_ids: None,
                    tags: None,
                    user_id: "".to_string(),
                })
            });

        warp::serve(warp::path("v1").and(quantum_processors.or(engagements)))
            .run(([127, 0, 0, 1], 8000))
            .await;
    }
}

#[allow(dead_code)]
mod qpu {
    use std::collections::HashMap;

    use log::{debug, error};
    use rmp_serde::Serializer;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    #[serde(tag = "_type")]
    #[allow(dead_code)]
    pub struct RPCRequest {
        method: String,
        params: Params,
        id: String,
        jsonrpc: String,
        client_timeout: u8,
        client_key: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum Params {
        QPUParams { request: QPURequest, priority: u8 },
        GetExecutionResultsRequest { job_id: String, wait: bool },
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "_type")]
    struct QPURequest {
        id: String,
        program: String,
        patch_values: HashMap<String, Vec<f64>>,
    }

    #[derive(Serialize, Debug)]
    #[serde(tag = "_type")]
    #[allow(dead_code)]
    pub enum RPCResponse<T> {
        RPCReply { id: String, result: T },
        RPCError { id: String, error: String },
    }

    #[derive(Serialize, Debug, PartialEq, Eq)]
    pub struct Buffer {
        shape: (usize,),
        data: [u8; 2],
        dtype: String,
    }

    #[derive(Serialize, Debug, PartialEq, Eq)]
    pub struct GetExecutionResultsResponse {
        buffers: HashMap<&'static str, Buffer>,
        execution_duration_microseconds: u64,
    }

    pub(crate) fn run() {
        let ctx = zmq::Context::new();

        let server = ctx.socket(zmq::ROUTER).unwrap();
        server.bind("tcp://127.0.0.1:8002").unwrap();

        loop {
            let (identity, data) = match server.recv_multipart(0) {
                Ok(mut parts) => {
                    if parts.len() != 2 {
                        error!("Invalid multipart data: {:?}", parts);
                        break;
                    }
                    let data = parts.pop().unwrap();
                    let identity = parts.pop().unwrap();
                    (identity, data)
                }
                Err(e) => {
                    error!("{}", e);
                    break;
                }
            };
            let response: Box<dyn erased_serde::Serialize> =
                match rmp_serde::from_read::<_, RPCRequest>(data.as_slice()) {
                    Ok(qpu_request) => match qpu_request.params {
                        Params::QPUParams { .. } => Box::new(RPCResponse::RPCReply {
                            id: qpu_request.id,
                            result: "1",
                        }),
                        Params::GetExecutionResultsRequest { .. } => {
                            let mut buffers = HashMap::new();
                            buffers.insert(
                                "q0",
                                Buffer {
                                    shape: (2,),
                                    data: [0, 1],
                                    dtype: "int8".to_string(),
                                },
                            );
                            buffers.insert(
                                "q1",
                                Buffer {
                                    shape: (2,),
                                    data: [0, 1],
                                    dtype: "int8".to_string(),
                                },
                            );
                            let resp = GetExecutionResultsResponse {
                                buffers,
                                execution_duration_microseconds: 8675,
                            };
                            Box::new(RPCResponse::RPCReply {
                                id: qpu_request.id,
                                result: resp,
                            })
                        }
                    },
                    Err(e) => {
                        debug!("{:?} could not be decoded with MsgPack:\n\t{}", data, e);
                        continue;
                    }
                };

            let mut response_buffer = vec![];
            response
                .serialize(&mut Serializer::new(&mut response_buffer).with_struct_map())
                .unwrap();
            server
                .send_multipart([identity, response_buffer], 0)
                .unwrap();
        }
    }
}
