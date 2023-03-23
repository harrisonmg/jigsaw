use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::Material2d,
};

#[derive(Debug, Clone, Default, ShaderType)]
pub struct PieceMaterialParams {
    pub sprite_origin: Vec2,
    pub sides: u32,
    pub _padding: u32,
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "77a244f3-9ff1-47e7-87de-97ffd4650eeb"]
pub struct PieceMaterial {
    #[uniform(0)]
    pub params: PieceMaterialParams,

    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for PieceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/piece.wgsl".into()
    }
}
