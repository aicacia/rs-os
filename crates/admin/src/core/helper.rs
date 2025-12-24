use std::hash::Hash;

use hashbrown::HashSet;

pub fn json_to_string_vec<T>(json_str: T) -> Vec<String>
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

pub fn string_vec_to_json(vec: &Vec<String>) -> String {
  match serde_json::to_string(vec) {
    Ok(json) => json,
    Err(e) => {
      log::error!("Error formatting JSON: {}", e);
      "[]".to_owned()
    }
  }
}

pub fn unordered_vec_equals<T>(a: &[T], b: &[T]) -> bool
where
  T: Hash + Eq,
{
  if a.len() != b.len() {
    return false;
  }
  let mut a_set = HashSet::with_capacity(a.len());
  a_set.extend(a.iter());

  let mut b_set = HashSet::with_capacity(b.len());
  b_set.extend(b.iter());

  a_set == b_set
}
