#![allow(clippy::module_inception)]
pub mod common;
pub mod entity;
pub mod router;
pub mod ws;

pub use os_api::{Form, Json, error};
pub use router::{ApiDoc, create_openapi_router};
