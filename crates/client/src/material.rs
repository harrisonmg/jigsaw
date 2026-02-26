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
    pub puzzle_texture: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub mask_texture: Handle<Image>,
    #[uniform(4)]
    pub uv_rect: Vec4,
}

impl Material2d for PieceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/piece.wgsl".into()
    }
}
