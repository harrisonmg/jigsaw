use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use game::Puzzle;

use crate::{
    colors::{LIGHT, MED},
    states::AppState,
    util::despawn,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(MED))
            .add_systems(Startup, add_board_assets)
            .add_systems(OnEnter(AppState::Cutting), spawn_board);
    }
}

#[derive(Resource)]
struct BoardMaterial(Handle<ColorMaterial>);

#[derive(Component)]
struct Board;

fn add_board_assets(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let material = materials.add(ColorMaterial::from(LIGHT));
    commands.insert_resource(BoardMaterial(material));
}

fn spawn_board(
    mut commands: Commands,
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    board_material: Res<BoardMaterial>,
    board_query: Query<Entity, With<Board>>,
) {
    despawn(board_query, &mut commands);

    let mesh = meshes.add(Mesh::from(Rectangle::new(
        puzzle.width() as f32,
        puzzle.height() as f32,
    )));

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: mesh.into(),
            material: board_material.0.clone(),
            ..default()
        })
        .insert(Board);
}
