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
    let qcs = Qcs::load();
    let endpoint = qcs.get_config().quilc_url();
    rpcq::Client::new(endpoint).unwrap()
}

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM).with_quilc_client(Some(quilc_client().await));
    let execution_options = ExecutionOptions::default();

    // You can submit a job to a QPU and retrieve results in a single step.
    let result = exe
        .with_parameter("theta", 0, PI)
        .execute_on_qpu("Ankaa-2", None, &execution_options)
        .await
        .expect("Program should execute successfully");

    dbg!(&result);

    // Or you can submit a job, then retrieve results later.
    let handle = exe
        .with_parameter("theta", 0, PI / 2.0)
        .submit_to_qpu("Ankaa-2", None, &execution_options)
        .await
        .expect("Program should be sumbitted successfully.");

    // You can use the job handle to attempt job cancellation. This will only succeed if the job
    // has not begun executing.
    let cancelled_result = exe.cancel_qpu_job(handle.clone()).await;
    dbg!(&cancelled_result);
    let cancelled = cancelled_result.is_ok();

    // Retrieving results will return an error if the job was successfully cancelled.
    let result = exe.retrieve_results(handle.clone()).await;
    if !cancelled {
        println!("Job was not cancelled");
        assert!(result.is_ok());
    } else {
        println!("Job was cancelled");
        assert!(result.is_err());
    }
    dbg!(&result);
}
