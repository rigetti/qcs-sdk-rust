use std::f64::consts::PI;

use qcs::Executable;

const PROGRAM: &str = r#"
DECLARE ro BIT[2]
DECLARE theta REAL
RX(theta) 0
CNOT 0 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"#;

fn quilc_client() -> qcs::compiler::libquil::Client {
    qcs::compiler::libquil::Client {}
}

fn qvm_client() -> qcs::qvm::libquil::Client {
    qcs::qvm::libquil::Client {}
}

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM).with_quilc_client(Some(quilc_client()));

    let result = exe
        .with_parameter("theta", 0, PI)
        .execute_on_qvm(&qvm_client())
        .await
        .expect("Program should execute successfully");

    println!("{result:?}");
}
