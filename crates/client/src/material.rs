use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(AsBindGroup, TypePath, TypeUuid, Debug, Clone)]
#[uuid = "77a244c3-9ff1-47e7-87de-97ffd4650eeb"]
pub struct PieceMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for PieceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/piece.wgsl".into()
    }
}
