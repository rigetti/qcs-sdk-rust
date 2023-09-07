//! This integration test implements the [Parametric Compilation example from pyQuil](example).
//! [example]: https://pyquil-docs.rigetti.com/en/stable/basics.html?highlight=parametric#parametric-compilation

use std::f64::consts::PI;

use qcs::{client::Qcs, compiler::rpcq, qvm, Executable};

const BASIC_SUBSTITUTION: &str = r#"
DECLARE ro BIT
DECLARE theta REAL

RX(pi / 2) 0
RZ(theta) 0
RX(-pi / 2) 0

MEASURE 0 ro[0]
"#;

async fn quilc_client() -> rpcq::Client {
    let qcs = Qcs::load().await;
    let endpoint = qcs.get_config().quilc_url();
    rpcq::Client::new(endpoint).unwrap()
}

async fn qvm_client() -> qvm::http::HttpClient {
    let qcs = Qcs::load().await;
    qvm::http::HttpClient::from(&qcs)
}

#[tokio::test]
async fn basic_substitution() {
    let mut exe = Executable::from_quil(BASIC_SUBSTITUTION)
        .with_quilc_client(Some(quilc_client().await))
        .with_qcs_client(Qcs::default());
    let qvm_client = qvm_client().await;
    let mut parametric_measurements = Vec::with_capacity(200);

    let step = 2.0 * PI / 200.0;

    for i in 0..=200 {
        let theta = step * f64::from(i);
        let result = exe
            .with_parameter("theta", 0, theta)
            .execute_on_qvm(&qvm_client)
            .await
            .expect("Executed on QPU");
        parametric_measurements.push(
            result
                .result_data
                .to_register_map()
                .expect("should convert to RegisterMap")
                .get_register_matrix("ro")
                .expect("should have `ro`")
                .as_integer()
                .expect("`ro` should have integer values")
                .get((0, 0))
                .expect("ro register should have a value in the first index and shot")
                .to_owned(),
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
