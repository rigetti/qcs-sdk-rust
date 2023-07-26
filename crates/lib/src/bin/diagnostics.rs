#[tokio::main]
async fn main() {
    let diagnostics = qcs::diagnostics::get_report().await;
    println!("{diagnostics}");
}
