//! This integration test implements the [Parametric Compilation example from pyQuil][example].
//! [example]: https://pyquil-docs.rigetti.com/en/stable/basics.html?highlight=parametric#parametric-compilation

use std::f64::consts::PI;

use qcs::Executable;

const BASIC_SUBSTITUTION: &str = r#"
DECLARE ro BIT
DECLARE theta REAL

RX(pi / 2) 0
RZ(theta) 0
RX(-pi / 2) 0

MEASURE 0 ro[0]
"#;

#[tokio::test]
async fn basic_substitution() {
    let mut exe = Executable::from_quil(BASIC_SUBSTITUTION);
    let mut parametric_measurements = Vec::with_capacity(200);

    let step = 2.0 * PI / 200.0;

    for i in 0..=200 {
        let theta = step * f64::from(i);
        let mut result = exe
            .with_parameter("theta", 0, theta)
            .execute_on_qvm()
            .await
            .expect("Executed on QPU");
        parametric_measurements.append(
            &mut result
                .registers
                .remove("ro")
                .expect("Found ro register")
                .into_i8()
                .unwrap()[0],
        )
    }

    for measurement in parametric_measurements {
        if measurement == 1 {
            // We didn't run with all 0 so parametrization worked!
            return;
        }
    }
    panic!("Results were all 0, parametrization must not have worked!");
}
