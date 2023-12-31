#[tokio::main]
async fn main() {
    env_logger::init();
    let diagnostics = qcs::diagnostics::get_report().await;
    println!("{diagnostics}");
}
