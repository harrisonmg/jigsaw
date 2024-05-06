use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
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
