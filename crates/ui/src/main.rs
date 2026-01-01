#[tokio::main]
async fn main() -> std::io::Result<()> {
  os_ui::cli::run::run().await
}
