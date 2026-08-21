//! These are the integration tests for [`qcs::Executable::execute_on_qvm`].
//!
//! The HTTP cases require quilc and QVM's web servers to be running, at localhost:5555
//! and localhost:5000 respectively. The libquil cases, built with the `libquil` feature,
//! call the linked library instead and need no servers.

use std::num::NonZeroU16;

use qcs::{client::Qcs, compiler::quilc, compiler::rpcq, qvm, Executable};

const PROGRAM: &str = r##"
DECLARE first BIT
DECLARE second BIT

H 0
CNOT 0 1

MEASURE 0 first
MEASURE 1 second
"##;

fn rpcq_quilc_client() -> rpcq::Client {
    let qcs = Qcs::load();
    let endpoint = qcs.get_config().quilc_url();
    rpcq::Client::new(endpoint).unwrap()
}

fn http_qvm_client() -> qvm::http::HttpClient {
    let qcs = Qcs::load();
    qvm::http::HttpClient::from(&qcs)
}

#[cfg(feature = "libquil")]
fn libquil_quilc_client() -> qcs::compiler::libquil::Client {
    qcs::compiler::libquil::Client {}
}

#[cfg(feature = "libquil")]
fn libquil_qvm_client() -> qvm::libquil::Client {
    qvm::libquil::Client {}
}

#[cfg_attr(feature = "libquil", test_case::test_case(libquil_quilc_client(), libquil_qvm_client() ; "with libquil clients"))]
#[test_case::test_case(rpcq_quilc_client(), http_qvm_client() ; "with server clients")]
#[tokio::test]
async fn test_bell_state<Q: quilc::Client + Send + Sync + 'static, V: qvm::Client>(
    quilc_client: Q,
    qvm_client: V,
) {
    let shots: NonZeroU16 = NonZeroU16::new(10).expect("value is non-zero");

    let data = Executable::from_quil(PROGRAM)
        .with_quilc_client(Some(quilc_client))
        .with_qcs_client(Qcs::load())
        .with_shots(shots)
        .read_from("first")
        .read_from("second")
        .execute_on_qvm(&qvm_client)
        .await
        .expect("Could not run on QVM");

    let first = data
        .result_data
        .to_register_map()
        .expect("should convert to readout map")
        .get_register_matrix("first")
        .expect("should have first register")
        .as_integer()
        .expect("first register should be integers")
        .to_owned();

    let second = data
        .result_data
        .to_register_map()
        .expect("should convert to readout map")
        .get_register_matrix("second")
        .expect("should have second register")
        .as_integer()
        .expect("second register should be integers")
        .to_owned();

    assert_eq!(first.shape(), [shots.get().into(), 1]);
    assert_eq!(second.shape(), [shots.get().into(), 1]);

    for (first, second) in first.into_iter().zip(second) {
        assert_eq!(first, second);
    }
}
