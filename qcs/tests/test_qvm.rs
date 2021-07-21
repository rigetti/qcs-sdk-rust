//! These are the integration tests for [`qvm_api::run_program_on_qvm`].
//! In order to run them, QVM's web server must be running at localhost:5000.

use eyre::Report;
use futures_retry::{ErrorHandler, RetryPolicy};
use qcs::qvm::*;
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
    const SHOTS: u16 = 10;

    // Sometimes the QVM container isn't ready yet when this runs, so let it retry
    let fut = futures_retry::FutureRetry::new(
        || run_program(PROGRAM, SHOTS, "ro"),
        RetryHandler { max_attempts: 5 },
    );

    let data = fut.await.expect("Could not run on QVM").0;

    for shot in data.into_i8().unwrap() {
        assert_eq!(shot.len(), 2);
        assert_eq!(shot[0], shot[1]);
    }
}

pub struct RetryHandler {
    max_attempts: usize,
}

impl ErrorHandler<Report> for RetryHandler {
    type OutError = Report;

    fn handle(&mut self, attempt: usize, e: Report) -> RetryPolicy<Report> {
        if attempt == self.max_attempts {
            eprintln!("Timed out talking to QVM");
            return RetryPolicy::ForwardError(e);
        }
        RetryPolicy::WaitRetry(Duration::from_secs(5))
    }
}
