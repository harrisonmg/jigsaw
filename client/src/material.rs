use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::Material2d,
};

#[derive(ShaderType, Debug, Clone, Default)]
pub struct PieceMaterialParams {
    pub sprite_origin: Vec2,
    pub sides: u32,
    pub _padding_1: u32,
    pub _padding_2: u32,
    pub _padding_3: u32,
    pub _padding_4: u32,
    pub _padding_5: u32,
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "77a244b3-9ff1-47e7-87de-97ffd4650eeb"]
pub struct PieceMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,

    #[uniform(2)]
    pub params: PieceMaterialParams,
}

impl Material2d for PieceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/piece.wgsl".into()
    }
}
