use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, Shader, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

const PIECE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 0x77a244c39ff147e7);

pub struct PieceMaterialPlugin;

impl Plugin for PieceMaterialPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.resource_mut::<Assets<Shader>>();
        shaders.set_untracked(
            PIECE_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../assets/shaders/piece.wgsl")),
        );
        app.add_plugins(Material2dPlugin::<PieceMaterial>::default());
    }
}

#[derive(AsBindGroup, TypePath, TypeUuid, Debug, Clone)]
#[uuid = "77a244c3-9ff1-47e7-87de-97ffd4650eeb"]
pub struct PieceMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub puzzle_texture: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub mask_texture: Handle<Image>,
}

impl Material2d for PieceMaterial {
    fn fragment_shader() -> ShaderRef {
        PIECE_SHADER_HANDLE.typed().into()
    }
}
