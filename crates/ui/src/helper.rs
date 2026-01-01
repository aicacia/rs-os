use std::{collections::HashMap, io, path::Path};

use crate::config::AppConfig;

pub async fn write_public_env_file(app_config: &AppConfig) -> io::Result<()> {
  let mut env_vars: HashMap<String, String> = std::env::vars()
    .filter(|(key, _)| key.starts_with("PUBLIC_"))
    .collect();

  if !env_vars.contains_key("PUBLIC_URL") {
    if let Some(url) = &app_config.url {
      env_vars.insert("PUBLIC_URL".to_owned(), url.to_owned());
    }
  }

  let env_body = serde_json::to_string(&env_vars)
    .map_err(|e| io::Error::other(format!("failed to serialize env vars: {}", e)))?;

  let env_path = Path::new(&app_config.static_dir).join(&app_config.env_file);

  log::info!("writing public env file to {:?}", env_path);
  tokio::fs::write(&env_path, format!("export const env={};", env_body))
    .await
    .map_err(|e| io::Error::other(format!("failed to write env file {:?}: {}", env_path, e)))
}
