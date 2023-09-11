use std::f64::consts::PI;

use qcs::{client::Qcs, compiler::rpcq, qpu::api::ExecutionOptions, Executable};

const PROGRAM: &str = r#"
DECLARE ro BIT[2]
DECLARE theta REAL
RX(theta) 0
CNOT 0 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"#;

async fn quilc_client() -> rpcq::Client {
    let qcs = Qcs::load().await;
    let endpoint = qcs.get_config().quilc_url();
    rpcq::Client::new(endpoint).unwrap()
}

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM).with_quilc_client(Some(quilc_client().await));

    let result = exe
        .with_parameter("theta", 0, PI)
        .execute_on_qpu("Aspen-M-3", None, &ExecutionOptions::default())
        .await
        .expect("Program should execute successfully");

    dbg!(&result);
}
