use std::collections::VecDeque;

use bevy::{
    prelude::*, render::mesh::VertexAttributeValues, sprite::MaterialMesh2dBundle, utils::HashMap,
};

use bevy_tweening::Animator;
use game::{Piece, PieceIndex, PieceMovedEvent, Puzzle};

use crate::{
    animation::new_piece_animator, better_quad::BetterQuad, material::PieceMaterial,
    states::AppState,
};

const MIN_PIECE_HEIGHT: f32 = 500.0;
const MAX_PIECE_HEIGHT: f32 = 900.0;

pub struct PiecePlugin;

impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PieceMovedEvent>()
            .add_systems(OnEnter(AppState::Setup), piece_setup)
            .add_systems(Update, move_piece.run_if(in_state(AppState::Playing)))
            .add_systems(Update, sort_pieces.run_if(in_state(AppState::Playing)));
    }
}

#[derive(Component)]
pub struct PieceComponent {
    index: PieceIndex,
    sprite_size: Vec2,
    sprite_origin: Vec2,
}

impl PieceComponent {
    pub fn index(&self) -> PieceIndex {
        self.index
    }

    pub fn within_sprite_bounds(&self, mut coords: Vec2) -> bool {
        coords += self.sprite_origin;
        0.0 <= coords.x
            && coords.x <= self.sprite_size.x
            && 0.0 <= coords.y
            && coords.y <= self.sprite_size.y
    }
}

#[derive(Bundle)]
pub struct PieceBundle {
    pub piece: PieceComponent,
    pub animator: Animator<Transform>,

    #[bundle]
    mesh_bundle: MaterialMesh2dBundle<PieceMaterial>,
}

impl PieceBundle {
    pub fn new(
        piece: &Piece,
        image_assets: &mut ResMut<Assets<Image>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<PieceMaterial>>,
    ) -> Self {
        let sprite = piece.sprite_clone();

        let sprite_size = Vec2::new(sprite.width() as f32, sprite.height() as f32);

        let sprite_origin = Vec2::new(
            piece.sprite_origin_x() as f32,
            piece.sprite_origin_y() as f32,
        );

        let piece_component = PieceComponent {
            index: piece.index(),
            sprite_size,
            sprite_origin,
        };

        let mut mesh = Mesh::from(BetterQuad::new(sprite_size, sprite_origin));

        let x_offset = 0.5 - sprite_origin.x / sprite_size.x;
        let y_offset = 0.5 - (sprite_size.y - sprite_origin.y) / sprite_size.y;
        let new_vertices = if let VertexAttributeValues::Float32x3(vertices) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        {
            vertices
                .iter()
                .map(|vertex| {
                    let mut points = *vertex;
                    points[0] += x_offset;
                    points[1] += y_offset;
                    points
                })
                .collect::<Vec<[f32; 3]>>()
        } else {
            panic!();
        };
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_vertices);

        let mesh_handle = meshes.add(mesh);
        let material = materials.add(PieceMaterial {
            texture: image_assets.add(sprite.into()),
        });

        let mut transform = piece.transform();
        transform.translation.z = MIN_PIECE_HEIGHT;
        let mesh_bundle = MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material,
            transform,
            ..Default::default()
        };

        Self {
            piece: piece_component,
            animator: new_piece_animator(),
            mesh_bundle,
        }
    }
}

#[derive(Resource)]
pub struct PieceMap(pub HashMap<PieceIndex, Entity>);

#[derive(Resource)]
pub struct PieceStack(pub VecDeque<Entity>);

impl PieceStack {
    fn remove_entity(&mut self, entity: Entity) {
        self.0
            .remove(self.0.iter().position(|e| *e == entity).unwrap());
    }

    pub fn put_on_top(&mut self, entity: Entity) {
        self.remove_entity(entity);
        self.0.push_front(entity);
    }

    pub fn put_on_bottom(&mut self, entity: Entity) {
        self.remove_entity(entity);
        self.0.push_back(entity);
    }
}

#[derive(Resource)]
pub struct HeldPiece {
    pub index: PieceIndex,
    pub cursor_offset: Vec2,
}

fn piece_setup(
    puzzle: Res<Puzzle>,
    mut image_assets: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PieceMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    let mut piece_map = PieceMap(HashMap::new());
    let mut piece_stack = PieceStack(VecDeque::new());

    puzzle.with_pieces(|piece| {
        let piece_bundle = PieceBundle::new(piece, &mut image_assets, &mut meshes, &mut materials);
        let piece_entity = commands.spawn(piece_bundle).id();

        let shadow_sprite = piece.shadow_sprite_clone();
        let shadow_x_offset = shadow_sprite.width() as f32 / 2.0 - piece.shadow_origin_x() as f32;
        let shadow_y_offset = shadow_sprite.height() as f32 / 2.0 - piece.shadow_origin_y() as f32;
        let shadow = SpriteBundle {
            transform: Transform::from_xyz(shadow_x_offset, shadow_y_offset, -MIN_PIECE_HEIGHT),
            texture: image_assets.add(piece.shadow_sprite_clone().into()),
            ..Default::default()
        };
        let shadow_entity = commands.spawn(shadow).id();

        commands
            .entity(piece_entity)
            .push_children(&[shadow_entity]);

        piece_map.0.insert(piece.index(), piece_entity);
        piece_stack.0.push_front(piece_entity);
    });

    commands.insert_resource(piece_map);
    commands.insert_resource(piece_stack);
    next_state.set(AppState::Playing);
}

fn move_piece(
    mut piece_moved_events: EventReader<PieceMovedEvent>,
    mut piece_query: Query<&mut Transform>,
    piece_map: Res<PieceMap>,
    puzzle: Res<Puzzle>,
    mut piece_stack: ResMut<PieceStack>,
) {
    for event in piece_moved_events.iter() {
        let piece_entity = *piece_map.0.get(&event.index).unwrap();
        let mut transform = piece_query.get_mut(piece_entity).unwrap();
        transform.translation.x = event.x;
        transform.translation.y = event.y;
        if puzzle.piece_group_locked(&event.index) {
            piece_stack.put_on_bottom(piece_entity);
        } else {
            piece_stack.put_on_top(piece_entity);
        }
    }
}

fn sort_pieces(
    mut piece_query: Query<&mut Transform, With<PieceComponent>>,
    piece_stack: Res<PieceStack>,
) {
    let piece_count = piece_query.iter().len();
    let z_step = (MAX_PIECE_HEIGHT - MIN_PIECE_HEIGHT) / piece_count as f32;
    for (i, piece_entity) in piece_stack.0.iter().enumerate() {
        let mut piece_transform = piece_query.get_mut(*piece_entity).unwrap();
        piece_transform.translation.z = MAX_PIECE_HEIGHT - i as f32 * z_step;
    }
}
