#![allow(clippy::module_inception)]
pub mod common;
pub mod entity;
pub mod middleware;
pub mod oidc;
pub mod router;

pub use os_api::{Form, Json, error};
pub use router::create_openapi_router;
