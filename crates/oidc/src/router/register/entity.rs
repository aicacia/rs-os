use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Validate, Deserialize, ToSchema)]
pub struct SignupRequest {
  #[validate(length(min = 1))]
  pub username: String,
  #[validate(length(min = 6), must_match(other = "password_confirmation"))]
  pub password: String,
  #[validate(length(min = 6))]
  pub password_confirmation: String,
  #[schema(example = "openid")]
  pub scope: Option<String>,
}
