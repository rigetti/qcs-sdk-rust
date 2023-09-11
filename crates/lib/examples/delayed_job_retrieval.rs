//! Submit a program to a QPU but don't immediately wait for the result.

use qcs::{qpu::api::ExecutionOptions, Executable};

const PROGRAM: &str = r#"
DECLARE ro BIT
RX(pi) 0
MEASURE 0 ro[0]
"#;

const QUANTUM_PROCESSOR_ID: &str = "Aspen-M-3";

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM);
    let job_handle = exe
        .submit_to_qpu(QUANTUM_PROCESSOR_ID, None, &ExecutionOptions::default())
        .await
        .expect("Program should be successfully submitted for execution");
    // Do some other stuff
    let _data = exe
        .retrieve_results(job_handle)
        .await
        .expect("Results should be successfully retrieved");
}
