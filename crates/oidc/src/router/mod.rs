pub mod client;
pub mod common;
pub mod current_user;
pub mod entity;
pub mod jwk;
pub mod middleware;
pub mod oidc;
pub mod openapi;
pub mod register;
mod router;
pub mod user;
pub mod user_email;
pub mod user_oauth2_provider;
pub mod user_phone_number;
pub mod user_role;
pub mod util;

// Re-export from os-api
pub use os_api::{error, Form, Json};

pub use router::{ApiDoc, create_router};
