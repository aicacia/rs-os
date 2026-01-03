#![allow(clippy::module_inception)]

pub mod client;
pub mod common;
pub mod current_user;
pub mod entity;
pub mod middleware;
pub mod router;
pub mod user;
pub mod user_email;
pub mod user_oauth2_provider;
pub mod user_phone_number;
pub mod user_role;

pub use router::create_openapi_router;
