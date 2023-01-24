//! These are the integration tests for [`qcs::Executable::execute_on_qvm`].
//! In order to run them, QVM's web server must be running at localhost:5000.

use qcs::Executable;
use qcs_api_client_openapi::common::ClientConfiguration;

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

    let data = Executable::from_quil(PROGRAM)
        .with_config(ClientConfiguration::default())
        .with_shots(SHOTS)
        .read_from("first")
        .read_from("second")
        .execute_on_qvm()
        .await
        .expect("Could not run on QVM");

    let first = data
        .readout_data
        .to_readout_map()
        .expect("should convert to readout map")
        .get_register_matrix("first")
        .expect("should have first register")
        .as_integer()
        .expect("first register should be integers")
        .to_owned();

    let second = data
        .readout_data
        .to_readout_map()
        .expect("should convert to readout map")
        .get_register_matrix("second")
        .expect("should have second register")
        .as_integer()
        .expect("second register should be integers")
        .to_owned();

    assert_eq!(first.shape(), [SHOTS.into(), 1]);
    assert_eq!(second.shape(), [SHOTS.into(), 1]);

    for (first, second) in first.into_iter().zip(second) {
        assert_eq!(first, second);
    }
}
