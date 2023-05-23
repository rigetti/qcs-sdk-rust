//! Integration tests for the [`qcs::qvm::api`] module. Requires the QVM
//! web server to be running.

use std::collections::HashMap;

use qcs::qvm::api;
use qcs_api_client_common::ClientConfiguration;
use regex::Regex;

const PROGRAM: &str = r##"
DECLARE ro BIT[2]
H 0
CNOT 0 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

#[tokio::test]
async fn test_get_version_info() {
    let config = ClientConfiguration::default();
    let version = api::get_version_info(&config)
        .await
        .expect("Should be able to get version info.");
    let semver_re = Regex::new(r"^([0-9]+)\.([0-9]+)\.([0-9]+)").unwrap();
    assert!(semver_re.is_match(&version))
}

#[tokio::test]
async fn test_run() {
    let config = ClientConfiguration::default();
    let request = api::MultishotRequest::new(
        PROGRAM.to_string(),
        2,
        HashMap::from([("ro".to_string(), api::AddressRequest::IncludeAll)]),
        Some((0.1, 0.5, 0.4)),
        Some((0.1, 0.5, 0.4)),
        Some(1),
    );
    let response = api::run(&request, &config)
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

#[tokio::test]
async fn test_run_and_measure() {
    let config = ClientConfiguration::default();
    let request = api::MultishotMeasureRequest::new(
        PROGRAM.to_string(),
        5,
        &[0, 1],
        Some((0.1, 0.5, 0.4)),
        Some((0.1, 0.5, 0.4)),
        Some(1),
    );
    let qubits = api::run_and_measure(&request, &config)
        .await
        .expect("Should be able to run and measure");
    assert_eq!(qubits.len(), 5);
    assert_eq!(qubits[0].len(), 2);
}

#[tokio::test]
async fn test_measure_expectation() {
    let config = ClientConfiguration::default();
    let prep_program = r##"
CSWAP 0 1 2
XY(-1.0) 0 1
Z 2
"##;
    let operators = vec!["X 0\nY 1\n".to_string(), "Z 2\n".to_string()];
    let request = api::ExpectationRequest::new(prep_program.to_string(), &operators, None);

    let expectations = api::measure_expectation(&request, &config)
        .await
        .expect("Should be able to measure expectation");

    assert_eq!(expectations.len(), operators.len());
}

#[tokio::test]
async fn test_get_wavefunction() {
    let config = ClientConfiguration::default();
    let request = api::WavefunctionRequest::new(PROGRAM.to_string(), None, None, Some(0));
    api::get_wavefunction(&request, &config)
        .await
        .expect("Should be able to get wavefunction");
}
