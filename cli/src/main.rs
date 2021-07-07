#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use qcs_api::apis::quantum_processors_api as api;
use qcs_util::get_configuration;

#[tokio::main]
async fn main() {
    let configuration = get_configuration().await.expect("Could not load config");
    let isa = api::get_instruction_set_architecture(configuration.as_ref(), "Aspen-9").await;
    println!("{:#?}", isa)
}
