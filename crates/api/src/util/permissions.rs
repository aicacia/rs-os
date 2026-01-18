/// Helper function to check if a user permission grants access to a required permission.
///
/// Supports wildcard matching:
/// - `*` grants all permissions
/// - `prefix:*` grants all permissions with that prefix (e.g., `client:*` grants `client:read`, `client:write`, etc.)
/// - Exact string matching for specific permissions
pub fn permission_grants(user_permission: &str, required_permission: &str) -> bool {
  if user_permission == "*" {
    return true;
  }

  if let Some(prefix) = user_permission.strip_suffix(":*") {
    if required_permission == prefix {
      return true;
    }
    if required_permission.starts_with(prefix)
      && required_permission.chars().nth(prefix.len()) == Some(':')
    {
      return true;
    }
  }

  user_permission == required_permission
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_permission_grants_exact() {
    assert!(permission_grants("fs:read", "fs:read"));
    assert!(!permission_grants("fs:read", "fs:write"));
  }

  #[test]
  fn test_permission_grants_wildcard_suffix() {
    assert!(permission_grants("fs:*", "fs:read"));
    assert!(permission_grants("fs:*", "fs:write"));
    assert!(permission_grants("os:*", "os:oidc"));
    assert!(permission_grants("os:*", "os:oidc:write"));
    assert!(!permission_grants("fs:*", "os:read"));
  }

  #[test]
  fn test_permission_grants_global_wildcard() {
    assert!(permission_grants("*", "fs:read"));
    assert!(permission_grants("*", "os:oidc:write"));
  }
}
