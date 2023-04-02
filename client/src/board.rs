use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use game::Puzzle;

use crate::{
    colors::{LIGHT, MED},
    states::AppState,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(MED))
            .add_systems(OnEnter(AppState::Setup), board_setup);
    }
}

fn board_setup(
    mut commands: Commands,
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let puzzle_size = Vec2::new(
        f32::from(puzzle.puzzle_width()) * puzzle.piece_width() as f32,
        f32::from(puzzle.puzzle_height()) * puzzle.piece_width() as f32,
    );
    let mesh = meshes.add(Mesh::from(shape::Quad::new(puzzle_size))).into();
    let material = materials.add(ColorMaterial::from(LIGHT));

    commands.spawn(MaterialMesh2dBundle {
        mesh,
        material,
        ..default()
    });
}
