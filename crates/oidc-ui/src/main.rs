#[tokio::main]
async fn main() -> std::io::Result<()> {
  os_oidc_ui::cli::run::run().await
}
