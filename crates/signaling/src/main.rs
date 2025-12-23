#[tokio::main]
#[cfg(feature = "bin")]
async fn main() -> std::io::Result<()> {
  os_signaling::cli::run::run().await
}

#[cfg(not(feature = "bin"))]
fn main() {
  println!("This crate is not built as a binary. Enable the 'bin' feature to build the binary.");
}
