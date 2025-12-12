use rust_embed::Embed;

#[derive(Embed, Clone)]
#[folder = "../frontend/build"]
pub struct Assets;
