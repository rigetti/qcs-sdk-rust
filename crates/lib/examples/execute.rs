use std::f64::consts::PI;

use qcs::Executable;

const PROGRAM: &str = r#"
DECLARE ro BIT[2]
DECLARE theta REAL
RX(theta) 0
X 0
CNOT 0 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"#;

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM);
    let theta = PI;

    let result = exe
        .with_parameter("theta", 0, theta)
        .execute_on_qpu("Aspen-11")
        .await
        .expect("Executed program on QPU");

    dbg!(&result);
}
