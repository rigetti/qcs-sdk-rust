//! These are the integration tests for [`qcs::Executable::execute_on_qvm`].
//! In order to run them, QVM's web server must be running at localhost:5000.

use qcs::Executable;

const PROGRAM: &str = r##"
DECLARE first BIT
DECLARE second BIT

H 0
CNOT 0 1

MEASURE 0 first
MEASURE 1 second
"##;

#[tokio::test]
async fn test_bell_state() {
    const SHOTS: u16 = 10;

    let mut data = Executable::from_quil(PROGRAM)
        .with_shots(SHOTS)
        .read_from("first")
        .read_from("second")
        .execute_on_qvm()
        .await
        .expect("Could not run on QVM");

    let first: Vec<Vec<i8>> = data
        .registers
        .remove("first")
        .expect("Missing first buffer")
        .into_i8()
        .expect("Produced wrong data type");
    let second: Vec<Vec<i8>> = data
        .registers
        .remove("second")
        .expect("Missing second buffer")
        .into_i8()
        .expect("Produced wrong data type");

    for (first, second) in first.into_iter().zip(second) {
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(first[0], second[0]);
    }
}
