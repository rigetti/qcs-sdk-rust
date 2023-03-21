//! Submit a program to a QPU but don't immediately wait for the result.

use qcs::Executable;

const PROGRAM: &str = r#"
DECLARE ro BIT
RX(pi) 0
MEASURE 0 ro[0]
"#;

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM);
    let quantum_processor_id = "Aspen-M-3";
    let job_handle = exe
        .submit_to_qpu(quantum_processor_id)
        .await
        .expect("Program should be successfully submitted for execution");
    // Do some other stuff
    let _data = exe
        .retrieve_results(quantum_processor_id, &job_handle)
        .await
        .expect("Results should be successfully retrieved");
}
