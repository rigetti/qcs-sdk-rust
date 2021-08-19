//! These are the integration tests for [`qcs::Executable::execute_on_qvm`].
//! In order to run them, QVM's web server must be running at localhost:5000.

use qcs::Executable;

const PROGRAM: &str = r##"
DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

#[tokio::test]
async fn test_bell_state() {
    const SHOTS: u16 = 10;

    let data = Executable::from_quil(PROGRAM)
        .with_shots(SHOTS)
        .read_from("ro")
        .execute_on_qvm()
        .await
        .expect("Could not run on QVM")
        .into_i8()
        .expect("Wrong data type returned");

    for shot in data {
        assert_eq!(shot.len(), 2);
        assert_eq!(shot[0], shot[1]);
    }
}
