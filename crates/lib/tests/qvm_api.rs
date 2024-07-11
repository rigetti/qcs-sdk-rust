//! Integration tests for the [`qcs::qvm::http`] module. Requires the QVM
//! web server to be running.

use std::{collections::HashMap, num::NonZeroU16};

use qcs::{
    client::Qcs,
    qvm::{
        self,
        http::{self, HttpClient},
        QvmOptions,
    },
};
use regex::Regex;

const PROGRAM: &str = r##"
DECLARE ro BIT[2]
I 0
X 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

async fn http_qvm_client() -> HttpClient {
    let qcs_client = Qcs::load();
    HttpClient::from(&qcs_client)
}

#[cfg(feature = "libquil")]
fn libquil_qvm_client() -> qvm::libquil::Client {
    qvm::libquil::Client {}
}

#[cfg_attr(feature = "libquil", test_case::test_case(libquil_qvm_client() ; "with libquil client"))]
#[test_case::test_case(http_qvm_client().await ; "with HTTP client")]
#[tokio::test]
async fn test_get_version_info<C: qvm::Client>(client: C) {
    let version = client
        .get_version_info(&QvmOptions::default())
        .await
        .expect("Should be able to get version info.");
    let semver_re = Regex::new(r"^([0-9]+)\.([0-9]+)\.([0-9]+)").unwrap();
    assert!(semver_re.is_match(&version))
}

#[cfg_attr(feature = "libquil", test_case::test_case(libquil_qvm_client() ; "with libquil client"))]
#[test_case::test_case(http_qvm_client().await ; "with HTTP client")]
#[tokio::test]
async fn test_run<C: qvm::Client>(client: C) {
    let request = http::MultishotRequest::new(
        PROGRAM.to_string(),
        NonZeroU16::new(2).expect("value is non-zero"),
        HashMap::from([("ro".to_string(), http::AddressRequest::IncludeAll)]),
        Some((0.1, 0.5, 0.4)),
        Some((0.1, 0.5, 0.4)),
        Some(1),
    );
    let response = client
        .run(&request, &QvmOptions::default())
        .await
        .expect("Should be able to run");
    assert_eq!(response.registers.len(), 1);
    let ro = response
        .registers
        .get("ro")
        .expect("Should get the 'ro' register back")
        .clone();
    assert_eq!(ro.into_i8().expect("A BIT register should be i8").len(), 2);
}

#[cfg_attr(feature = "libquil", test_case::test_case(libquil_qvm_client() ; "with libquil client"))]
#[test_case::test_case(http_qvm_client().await ; "with HTTP client")]
#[tokio::test]
async fn test_run_and_measure<C: qvm::Client>(client: C) {
    let request = http::MultishotMeasureRequest::new(
        PROGRAM.to_string(),
        NonZeroU16::new(5).expect("value is non-zero"),
        &[0, 1],
        Some((0.1, 0.5, 0.4)),
        Some((0.1, 0.5, 0.4)),
        Some(1),
    );
    let qubits = client
        .run_and_measure(&request, &QvmOptions::default())
        .await
        .expect("Should be able to run and measure");
    assert_eq!(qubits.len(), 5);
    assert_eq!(qubits[0].len(), 2);
}

#[cfg_attr(feature = "libquil", test_case::test_case(libquil_qvm_client() ; "with libquil client"))]
#[test_case::test_case(http_qvm_client().await ; "with HTTP client")]
#[tokio::test]
async fn test_measure_expectation<C: qvm::Client>(client: C) {
    let prep_program = r##"
CSWAP 0 1 2
XY(-1.0) 0 1
Z 2
"##;
    let operators = vec!["X 0\nY 1\n".to_string(), "Z 2\n".to_string()];
    let request = http::ExpectationRequest::new(prep_program.to_string(), &operators, None);

    let expectations = client
        .measure_expectation(&request, &QvmOptions::default())
        .await
        .expect("Should be able to measure expectation");

    assert_eq!(expectations.len(), operators.len());
}

#[cfg_attr(feature = "libquil", test_case::test_case(libquil_qvm_client() ; "with libquil client"))]
#[test_case::test_case(http_qvm_client().await ; "with HTTP client")]
#[tokio::test]
async fn test_get_wavefunction<C: qvm::Client>(client: C) {
    let request = http::WavefunctionRequest::new(PROGRAM.to_string(), None, None, Some(0));
    client
        .get_wavefunction(&request, &QvmOptions::default())
        .await
        .expect("Should be able to get wavefunction");
}
