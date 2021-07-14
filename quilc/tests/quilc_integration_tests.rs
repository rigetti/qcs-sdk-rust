use qcs_util::Configuration;
use quilc::compile_program;

mod isa_fixtures;

const EXPECTED_H0_OUTPUT: &str = r#"MEASURE 0                               # Entering rewiring: #(0 1)
HALT                                    # Exiting rewiring: #(0 1)
"#;

#[test]
fn compare_native_quil_to_expected_output() {
    let output = compile_program(
        "MEASURE 0",
        &isa_fixtures::qvm_2q(),
        &Configuration::default(),
    )
    .expect("Could not compile");
    assert_eq!(String::from(output), EXPECTED_H0_OUTPUT);
}

const BELL_STATE: &str = r##"DECLARE ro BIT[2]

H 0
CNOT 0 1

MEASURE 0 ro[0]
MEASURE 1 ro[1]
"##;

#[tokio::test]
async fn run_compiled_bell_state_on_qvm() {
    let config = Configuration::default();
    let output =
        compile_program(BELL_STATE, &isa_fixtures::aspen_9(), &config).expect("Could not compile");
    let results = qvm::run_program(&String::from(output), 10, "ro")
        .await
        .expect("Could not run program on QVM");
    for shot in results {
        assert_eq!(shot.len(), 2);
        assert_eq!(shot[0], shot[1]);
    }
}
