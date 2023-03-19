use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "77a244f3-9ff1-47e7-87de-97ffd4650eeb"]
pub struct PieceMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,

    #[uniform(2)]
    pub sprite_origin: Vec2,
    //#[uniform(3)]
    //pub sides: u32,
}

impl Material2d for PieceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/piece.wgsl".into()
    }
}
