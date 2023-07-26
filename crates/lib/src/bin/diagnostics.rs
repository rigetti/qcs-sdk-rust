#[tokio::main]
async fn main() {
    let diagnostics = qcs::diagnostics::gather().await;
    println!("{diagnostics}");
}
