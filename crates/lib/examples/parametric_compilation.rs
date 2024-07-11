//! This example implements the [Parametric Compilation example from pyQuil](pyquil) in Rust.
//! [pyquil]: https://pyquil-docs.rigetti.com/en/stable/basics.html?highlight=parametric#parametric-compilation

use std::f64::consts::PI;
use std::time::Duration;

use qcs::client::Qcs;
use qcs::compiler::rpcq;
use qcs::qpu::api::ExecutionOptions;
use qcs::Executable;

const PROGRAM: &str = r#"
DECLARE ro BIT
DECLARE theta REAL

RX(pi / 2) 0
RZ(theta) 0
RX(-pi / 2) 0

MEASURE 0 ro[0]
"#;
async fn quilc_client() -> rpcq::Client {
    let qcs = Qcs::load();
    let endpoint = qcs.get_config().quilc_url();
    rpcq::Client::new(endpoint).unwrap()
}

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM).with_quilc_client(Some(quilc_client().await));
    let mut parametric_measurements = Vec::with_capacity(200);

    let step = 2.0 * PI / 50.0;
    let mut total_execution_time = Duration::new(0, 0);

    for i in 0..=50 {
        let theta = step * f64::from(i);
        let data = exe
            .with_parameter("theta", 0, theta)
            .execute_on_qpu("Aspen-M-3", None, &ExecutionOptions::default())
            .await
            .expect("Executed program on QPU");
        total_execution_time += data
            .duration
            .expect("Aspen-M-3 should always report duration");

        let first_ro_values = data
            .result_data
            .to_register_map()
            .expect("should be able to create a RegisterMap")
            .get_register_matrix("ro")
            .expect("readout values should contain 'ro'")
            .as_integer()
            .expect("'ro' should be a register of integer values")
            .row(0)
            .to_owned();

        for value in &first_ro_values {
            parametric_measurements.push(*value)
        }
    }

    println!("Total execution time: {total_execution_time:?}");

    for measurement in parametric_measurements {
        if measurement == 1 {
            // We didn't run with all 0 so parametrization worked!
            return;
        }
    }
    panic!("Results were all 0, parametrization must not have worked!");
}
