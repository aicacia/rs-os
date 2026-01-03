#![allow(clippy::module_inception)]

pub mod entity;
pub mod fs;
pub mod router;

pub use router::create_openapi_router;
