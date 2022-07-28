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
    let job_handle = exe
        .submit_to_qpu("Aspen-11")
        .await
        .expect("Failed to submit");
    // Do some other stuff
    let _data = exe
        .retrieve_results(job_handle)
        .await
        .expect("Failed to retrieve results");
}
