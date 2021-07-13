use qcs_api::apis::quantum_processors_api as api;
use qcs_util::get_configuration;

#[tokio::main]
async fn main() {
    let config = get_configuration().await.expect("Could not load config");
    let isa = api::get_instruction_set_architecture(config.as_ref(), "Aspen-9")
        .await
        .expect("Could not fetch ISA from QCS");
    let _native_quil = quilc::compile_program("H 0", &isa, &config);
}
