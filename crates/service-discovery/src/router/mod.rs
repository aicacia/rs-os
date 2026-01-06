#![allow(clippy::module_inception)]

pub mod discovery;
pub mod entity;
pub mod router;

pub use os_api::{Form, Json, error};
pub use router::{ApiDoc, create_openapi_router};
