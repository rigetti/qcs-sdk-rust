use std::f64::consts::PI;

use qcs::{qpu::api::ConnectionStrategy, Executable};

const PROGRAM: &str = r#"
DECLARE ro BIT[2]
DECLARE theta REAL
RX(theta) 0
CNOT 0 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"#;

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM);

    let result = exe
        .with_parameter("theta", 0, PI)
        .execute_on_qpu("Aspen-M-3", None, ConnectionStrategy::default())
        .await
        .expect("Program should execute successfully");

    dbg!(&result);
}
