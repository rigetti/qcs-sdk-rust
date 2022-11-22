//! This example runs a basic [Quil-T program from pyQuil](pyquil) in Rust.
//!
//! [pyquil]: https://pyquil-docs.rigetti.com/en/stable/quilt_getting_started.html#Another-example:-a-simple-T1-experiment

use qcs::Executable;

/// This program doesn't do much, the main point is that it will fail if quilc is invoked.
const PROGRAM: &str = r#"
DECLARE ro BIT
RX(pi) 0
FENCE 0
DELAY 0 "rf" 1e-6
MEASURE 0 ro
"#;

#[tokio::main]
async fn main() {
    let exe = Executable::from_quil(PROGRAM);

    let result = exe
        .compile_with_quilc(false)
        .execute_on_qpu("Aspen-11")
        .await
        .expect("Program should execute successfully")
        .readout_data
        .get_values_by_memory_index("ro", 0)
        .expect("Readout data should include 'ro'");

    for value in result.iter() {
        value
            .expect("values should exist")
            .into_integer()
            .expect("should be integers");
    }
}
