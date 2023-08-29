//! This example runs a basic [Quil-T program from pyQuil](pyquil) in Rust.
//!
//! [pyquil]: https://pyquil-docs.rigetti.com/en/stable/quilt_getting_started.html#Another-example:-a-simple-T1-experiment

use qcs::{client::Qcs, compiler::rpcq, qpu::api::ExecutionOptions, Executable};

/// This program doesn't do much, the main point is that it will fail if quilc is invoked.
const PROGRAM: &str = r#"
DECLARE ro BIT
RX(pi) 0
FENCE 0
DELAY 0 "rf" 1e-6
MEASURE 0 ro
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
        .execute_on_qpu("Aspen-M-3", None, &ExecutionOptions::default())
        .await
        .expect("Program should execute successfully")
        .result_data
        .to_register_map()
        .expect("should be able to convert execution data to RegisterMap")
        .get_register_matrix("ro")
        .expect("Register data should include 'ro'")
        .as_integer()
        .expect("ro should be a register of integer values")
        .to_owned();

    println!("{result:?}");
}
