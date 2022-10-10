//! This example implements the [Parametric Compilation example from pyQuil](pyquil) in Rust.
//! [pyquil]: https://pyquil-docs.rigetti.com/en/stable/basics.html?highlight=parametric#parametric-compilation

use std::f64::consts::PI;
use std::time::Duration;

use qcs::Executable;
use qcs_api_client_grpc::models::controller::{readout_values::Values, IntegerReadoutValues};

const PROGRAM: &str = r#"
DECLARE ro BIT
DECLARE theta REAL

RX(pi / 2) 0
RZ(theta) 0
RX(-pi / 2) 0

MEASURE 0 ro[0]
"#;

#[tokio::main]
async fn main() {
    let mut exe = Executable::from_quil(PROGRAM);
    let mut parametric_measurements = Vec::with_capacity(200);

    let step = 2.0 * PI / 50.0;
    let mut total_execution_time = Duration::new(0, 0);

    for i in 0..=50 {
        let theta = step * f64::from(i);
        let data = exe
            .with_parameter("theta", 0, theta)
            .execute_on_qpu("Aspen-11")
            .await
            .expect("Executed program on QPU");
        total_execution_time += data
            .duration
            .expect("Aspen-11 should always report duration");

        let ro_readout_data = data
            .readout_data
            .get_readout_values_for_field("ro")
            .expect("readout values should contain 'ro'")
            .expect("'ro' should contain readout values");
        let first_ro_data = ro_readout_data
            .first()
            .expect("'ro' should contain at least one readout value")
            .clone();
        let first_ro_values = first_ro_data
            .expect("first readout value should ")
            .values
            .expect("'ro' should have readout values");
        if let Values::IntegerValues(IntegerReadoutValues { mut values }) = first_ro_values {
            parametric_measurements.append(&mut values)
        }
    }

    println!("Total execution time: {:?}", total_execution_time);

    for measurement in parametric_measurements {
        if measurement == 1 {
            // We didn't run with all 0 so parametrization worked!
            return;
        }
    }
    panic!("Results were all 0, parametrization must not have worked!");
}
