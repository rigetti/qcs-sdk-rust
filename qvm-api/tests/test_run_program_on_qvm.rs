//! These are the integration tests for [`qvm_api::run_program_on_qvm`].
//! In order to run them, QVM's web server must be running at localhost:5000.

use qvm_api::*;

const PROGRAM: &str = r##"
DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

#[tokio::test]
async fn test_bell_state() {
    let shots = 10;
    let mut response = run_program_on_qvm(PROGRAM, shots, "ro")
        .await
        .expect("Failed to run program");
    let data = response
        .registers
        .remove("ro")
        .expect("ro Register was missing from response");
    assert_eq!(data.len(), shots as usize);
    for shot in data {
        assert_eq!(shot.len(), 2);
        assert_eq!(shot[0], shot[1]);
    }
}
