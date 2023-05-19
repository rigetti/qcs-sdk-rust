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
        PROGRAM,
        2,
        HashMap::from([("ro".to_string(), api::AddressRequest::All(true))]),
        Some((0.1, 0.5, 0.4)),
        Some((0.1, 0.5, 0.4)),
        Some(1),
    );
    let response = api::run(&request, &config)
        .await
        .expect("Should be able to run");
    dbg!(&response);
    assert_eq!(response.registers.len(), 1);
    let ro = response
        .registers
        .get("ro")
        .expect("Should get the 'ro' register back")
        .clone();
    assert_eq!(ro.into_i8().expect("A BIT register should be i8").len(), 2);
}
