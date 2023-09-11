//! Use some local servers to stub out real requests to QCS in order to test the end to end flow of
//! the `qcs` crate.

use std::time::Duration;

use futures::future;
use ndarray::arr2;

use qcs::{
    client::Qcs,
    compiler::rpcq,
    qpu::api::{ConnectionStrategy, ExecutionOptionsBuilder},
    Executable,
};
use qcs_api_client_common::configuration::{SECRETS_PATH_VAR, SETTINGS_PATH_VAR};

const BELL_STATE: &str = r#"
DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"#;

const QPU_ID: &str = "Aspen-M-3";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_qcs_against_mocks() {
    // Shared setup
    setup().await;

    let mut handles = Vec::new();
    for _ in 0..3 {
        // Test direct access
        handles.push(tokio::spawn(run_bell_state(
            ConnectionStrategy::DirectAccess,
        )));
        // Check gateway access
        handles.push(tokio::spawn(run_bell_state(ConnectionStrategy::Gateway)));
    }

    // Ensure both access methods were cached
    future::try_join_all(handles).await.unwrap();
    assert_eq!(
        1,
        mock_qcs::DEFAULT_ENDPOINT_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
    );
    assert_eq!(
        1,
        mock_qcs::ACCESSORS_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
    );
}

async fn setup() {
    simple_logger::init_with_env().unwrap();
    std::env::set_var(SETTINGS_PATH_VAR, "tests/settings.toml");
    std::env::set_var(SECRETS_PATH_VAR, "tests/secrets.toml");
    tokio::spawn(qpu::run());
    tokio::spawn(translation::run());
    tokio::spawn(auth_server::run());
    tokio::spawn(mock_qcs::run());
}

async fn quilc_client() -> rpcq::Client {
    let qcs = Qcs::load().await;
    let endpoint = qcs.get_config().quilc_url();
    rpcq::Client::new(endpoint).unwrap()
}

async fn run_bell_state(connection_strategy: ConnectionStrategy) {
    let execution_options_direct_access = ExecutionOptionsBuilder::default()
        .connection_strategy(connection_strategy)
        .build()
        .expect("should be valid execution options");
    let result = Executable::from_quil(BELL_STATE)
        .with_quilc_client(Some(quilc_client().await))
        .with_shots(std::num::NonZeroU16::new(2).expect("value is non-zero"))
        .execute_on_qpu(QPU_ID, None, &execution_options_direct_access)
        .await
        .expect("Failed to run program that should be successful");
    assert_eq!(
        result
            .result_data
            .to_register_map()
            .expect("should convert to RegisterMap")
            .get_register_matrix("ro")
            .expect("should have values for `ro`")
            .as_integer()
            .expect("`ro` should have integer values"),
        arr2(&[[0, 1], [0, 1],]),
    );
    assert_eq!(result.duration, Some(Duration::from_micros(8675)));
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
    use serde_json::json;
    use warp::Filter;

    use qcs_api_client_openapi::models::{
        InstructionSetArchitecture, ListQuantumProcessorAccessorsResponse,
        QuantumProcessorAccessor, QuantumProcessorAccessorType,
        TranslateNativeQuilToEncryptedBinaryRequest, TranslateNativeQuilToEncryptedBinaryResponse,
    };

    const MOCK_QPU_ADDRESS: &str = "http://127.0.0.1:8002";
    pub(crate) static DEFAULT_ENDPOINT_CALL_COUNT: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);
    pub(crate) static ACCESSORS_CALL_COUNT: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);

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

        let translate = warp::path(format!("{QPU_ID}:translateNativeQuilToEncryptedBinary"))
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

        use std::sync::atomic::Ordering::SeqCst;
        let default_endpoint = warp::path(QPU_ID)
            .and(warp::path("endpoints:getDefault"))
            .and(warp::get())
            .map(|| {
                DEFAULT_ENDPOINT_CALL_COUNT.fetch_add(1, SeqCst);
                let endpoint = json!({
                    "address": "",
                    "addresses": {
                        "grpc": MOCK_QPU_ADDRESS,
                    },
                    "datacenter": "west-1",
                    "healthy": true,
                    "id": QPU_ID.to_string(),
                    "mock": true,
                    "quantumProcessorIds": [QPU_ID.to_string()],
                });
                warp::reply::json(&endpoint)
            });

        let accessors = warp::path(QPU_ID)
            .and(warp::path("accessors"))
            .and(warp::get())
            .map(|| {
                ACCESSORS_CALL_COUNT.fetch_add(1, SeqCst);
                let rsp = ListQuantumProcessorAccessorsResponse {
                    accessors: vec![QuantumProcessorAccessor {
                        access_type: Some(Box::new(QuantumProcessorAccessorType::GatewayV1)),
                        live: true,
                        rank: Some(0),
                        id: Some(QPU_ID.to_string()),
                        url: MOCK_QPU_ADDRESS.into(),
                    }],
                    next_page_token: None,
                };
                warp::reply::json(&rsp)
            });

        let quantum_processors = warp::path("quantumProcessors")
            .and(isa.or(translate).or(default_endpoint).or(accessors));

        warp::serve(warp::path("v1").and(quantum_processors))
            .run(([127, 0, 0, 1], 8000))
            .await;
    }
}

mod translation {
    use std::collections::HashMap;

    use qcs_api_client_grpc::models::controller::EncryptedControllerJob;
    use qcs_api_client_grpc::models::translation::QuilTranslationMetadata;
    use qcs_api_client_grpc::services::translation::translation_server::{
        Translation, TranslationServer,
    };
    use qcs_api_client_grpc::services::translation::{
        TranslateQuilToEncryptedControllerJobRequest, TranslateQuilToEncryptedControllerJobResponse,
    };
    use tonic::{transport::Server, Request};
    use tonic::{Response, Status};

    #[derive(Default, Debug)]
    pub struct TranslationService {}

    #[tonic::async_trait]
    impl Translation for TranslationService {
        async fn translate_quil_to_encrypted_controller_job(
            &self,
            _request: Request<TranslateQuilToEncryptedControllerJobRequest>,
        ) -> Result<Response<TranslateQuilToEncryptedControllerJobResponse>, Status> {
            Ok(Response::new(
                TranslateQuilToEncryptedControllerJobResponse {
                    job: Some(EncryptedControllerJob {
                        job: Vec::new(),
                        encryption: None,
                    }),
                    metadata: Some(QuilTranslationMetadata {
                        readout_mappings: HashMap::from([
                            ("ro[0]".to_string(), "q0".to_string()),
                            ("ro[1]".to_string(), "q1".to_string()),
                        ]),
                    }),
                },
            ))
        }
    }

    pub(crate) async fn run() {
        let service = TranslationService::default();
        Server::builder()
            .add_service(TranslationServer::new(service))
            .serve("127.0.0.1:8003".parse().expect("address can be parsed"))
            .await
            .expect("service runs without errors");
    }
}

mod qpu {
    use std::collections::HashMap;

    use qcs_api_client_grpc::{
        models::controller::{
            readout_values::Values, ControllerJobExecutionResult, IntegerReadoutValues,
            ReadoutValues,
        },
        services::controller::{
            controller_server::{Controller, ControllerServer},
            BatchExecuteControllerJobsRequest, BatchExecuteControllerJobsResponse,
            CancelControllerJobsRequest, CancelControllerJobsResponse, ExecuteControllerJobRequest,
            ExecuteControllerJobResponse, GetControllerJobResultsRequest,
            GetControllerJobResultsResponse, GetControllerJobStatusRequest,
            GetControllerJobStatusResponse,
        },
    };
    use tonic::{transport::Server, Request, Response, Status};

    #[derive(Default, Debug)]
    pub struct ControllerService {}

    #[tonic::async_trait]
    impl Controller for ControllerService {
        async fn execute_controller_job(
            &self,
            _request: Request<ExecuteControllerJobRequest>,
        ) -> Result<Response<ExecuteControllerJobResponse>, Status> {
            Ok(Response::new(ExecuteControllerJobResponse {
                job_execution_ids: vec!["job-id".to_string()],
            }))
        }

        async fn batch_execute_controller_jobs(
            &self,
            _request: Request<BatchExecuteControllerJobsRequest>,
        ) -> Result<Response<BatchExecuteControllerJobsResponse>, Status> {
            unimplemented!()
        }

        async fn get_controller_job_results(
            &self,
            _request: Request<GetControllerJobResultsRequest>,
        ) -> Result<Response<GetControllerJobResultsResponse>, Status> {
            Ok(Response::new(GetControllerJobResultsResponse {
                result: Some(ControllerJobExecutionResult {
                    memory_values: HashMap::new(),
                    readout_values: HashMap::from([
                        (
                            "q0".to_string(),
                            ReadoutValues {
                                values: Some(Values::IntegerValues(IntegerReadoutValues {
                                    values: vec![0, 0],
                                })),
                            },
                        ),
                        (
                            "q1".to_string(),
                            ReadoutValues {
                                values: Some(Values::IntegerValues(IntegerReadoutValues {
                                    values: vec![1, 1],
                                })),
                            },
                        ),
                    ]),
                    status: 1,
                    status_message: Some("success".to_string()),
                    execution_duration_microseconds: 8675,
                }),
            }))
        }

        async fn cancel_controller_jobs(
            &self,
            _request: Request<CancelControllerJobsRequest>,
        ) -> Result<Response<CancelControllerJobsResponse>, Status> {
            unimplemented!()
        }

        async fn get_controller_job_status(
            &self,
            _request: Request<GetControllerJobStatusRequest>,
        ) -> Result<Response<GetControllerJobStatusResponse>, Status> {
            unimplemented!()
        }
    }

    pub(crate) async fn run() {
        let service = ControllerService::default();
        Server::builder()
            .add_service(ControllerServer::new(service))
            // port must match MOCK_QPU_ADDRESS
            .serve("127.0.0.1:8002".parse().expect("address can be parsed"))
            .await
            .expect("service can be awaited");
    }
}
