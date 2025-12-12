#[tokio::main]
async fn main() -> std::io::Result<()> {
  os_admin_ui::cli::run::run().await
}
