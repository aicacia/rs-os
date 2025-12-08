pub mod constants;
pub mod entity;
pub mod router;

pub use constants::{DESCRIPTION, TAG};
pub use entity::{Health, Version};
pub use router::create_router;
