#[tokio::main]
async fn main() -> std::io::Result<()> {
  os::cli::run::run().await
}
