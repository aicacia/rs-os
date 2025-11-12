pub fn json_to_string_array<T>(json_str: T) -> Vec<String>
where
  T: AsRef<str>,
{
  match serde_json::from_str(json_str.as_ref()) {
    Ok(value) => value,
    Err(e) => {
      log::error!("Error parsing JSON: {}", e);
      Vec::default()
    }
  }
}
