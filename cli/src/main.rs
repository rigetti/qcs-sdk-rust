#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]

use qcs_api::apis::quantum_processors_api;
use qcs_util::get_configuration;

#[tokio::main]
async fn main() {
    let configuration = get_configuration().await.expect("Could not load config");
    let qpus = quantum_processors_api::list_quantum_processors(&configuration, None, None).await;
    println!("{:#?}", qpus)
}
