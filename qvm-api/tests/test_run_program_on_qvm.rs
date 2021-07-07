//! These are the integration tests for [`qvm_api::run_program_on_qvm`].
//! In order to run them, QVM's web server must be running at localhost:5000.

use futures_retry::{ErrorHandler, RetryPolicy};
use qvm_api::*;
use std::time::Duration;

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
    let config = qcs_util::Configuration::default();

    // Sometimes the QVM container isn't ready yet when this runs, so let it retry
    let fut = futures_retry::FutureRetry::new(
        || run_program_on_qvm(PROGRAM, shots, "ro", &config),
        RetryHandler { max_attempts: 5 },
    );

    let mut response = fut.await.expect("Could not run on QVM").0;
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

pub struct RetryHandler {
    max_attempts: usize,
}

impl ErrorHandler<QVMError> for RetryHandler {
    type OutError = QVMError;

    fn handle(&mut self, attempt: usize, e: QVMError) -> RetryPolicy<QVMError> {
        if attempt == self.max_attempts {
            eprintln!("Timed out talking to QVM");
            return RetryPolicy::ForwardError(e);
        }
        match e {
            QVMError::ConnectionError(_) => RetryPolicy::WaitRetry(Duration::from_secs(5)),
        }
    }
}
