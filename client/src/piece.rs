use bevy::{
    prelude::*, render::mesh::VertexAttributeValues, sprite::MaterialMesh2dBundle, utils::HashMap,
};

use game::{Piece, PieceIndex, PieceMoved, Puzzle};

use crate::{better_quad::BetterQuad, material::PieceMaterial, states::AppState};

const MIN_PIECE_HEIGHT: f32 = 500.0;
const MAX_PIECE_HEIGHT: f32 = 900.0;

pub struct PiecePlugin;

impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PieceMoved>()
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
    pub stack_pos: usize,
}

impl PieceComponent {
    pub fn index(&self) -> PieceIndex {
        self.index
    }

    pub fn within_sprite_bounds(&self, mut coords: Vec2) -> bool {
        coords += self.sprite_origin;
        return 0.0 <= coords.x
            && coords.x <= self.sprite_size.x
            && 0.0 <= coords.y
            && coords.y <= self.sprite_size.y;
    }
}

#[derive(Bundle)]
pub struct PieceBundle {
    pub piece: PieceComponent,

    #[bundle]
    mesh_bundle: MaterialMesh2dBundle<PieceMaterial>,
}

impl PieceBundle {
    pub fn new(
        piece: &Piece,
        stack_pos: usize,
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
            stack_pos,
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

        Self {
            piece: piece_component,
            mesh_bundle: MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material,
                transform,
                ..Default::default()
            },
        }
    }
}

#[derive(Resource)]
pub struct PieceMap(pub HashMap<PieceIndex, Entity>);

#[derive(Resource)]
pub struct PieceStack(pub Vec<Entity>);

impl PieceStack {
    pub fn put_on_top(&mut self, piece: &mut PieceComponent, entity: Entity) {
        piece.stack_pos = self.0.len();
        self.0.push(entity);
    }
}

#[derive(Resource)]
pub struct HeldPiece(pub PieceIndex);

fn piece_setup(
    mut commands: Commands,
    puzzle: Res<Puzzle>,
    mut image_assets: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PieceMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut piece_map = PieceMap(HashMap::new());
    let mut piece_stack = PieceStack(Vec::new());

    let mut i = 0;
    puzzle.with_pieces(|piece| {
        let piece_bundle =
            PieceBundle::new(piece, i, &mut image_assets, &mut meshes, &mut materials);
        let piece_entity = commands.spawn(piece_bundle).id();

        let shadow_sprite = piece.shadow_sprite_clone();
        let shadow_x_offset = shadow_sprite.width() as f32 / 2.0 - piece.shadow_origin_x() as f32;
        let shadow_y_offset = shadow_sprite.height() as f32 / 2.0 - piece.shadow_origin_y() as f32;
        let shadow = SpriteBundle {
            transform: Transform::from_xyz(
                shadow_x_offset + 10.0,
                shadow_y_offset + 10.0,
                -MIN_PIECE_HEIGHT + 1.0,
            ),
            texture: image_assets.add(piece.shadow_sprite_clone().into()),
            ..Default::default()
        };
        let shadow_entity = commands.spawn(shadow).id();

        commands
            .entity(piece_entity)
            .push_children(&[shadow_entity]);

        piece_map.0.insert(piece.index(), piece_entity);
        piece_stack.0.push(piece_entity);
        i += 1;
    });

    commands.insert_resource(piece_map);
    commands.insert_resource(piece_stack);
    next_state.set(AppState::Playing);
}

fn move_piece(
    mut piece_moved_events: EventReader<PieceMoved>,
    mut piece_query: Query<(&mut Transform, &mut PieceComponent)>,
    piece_map: Res<PieceMap>,
    mut piece_stack: ResMut<PieceStack>,
) {
    for event in piece_moved_events.iter() {
        let piece_entity = *piece_map.0.get(&event.index).unwrap();
        let (mut transform, mut piece) = piece_query.get_mut(piece_entity).unwrap();
        transform.translation.x = event.x;
        transform.translation.y = event.y;
        transform.rotation = Quat::from_rotation_z(event.rotation);
        piece_stack.put_on_top(&mut piece, piece_entity);
    }
}

fn sort_pieces(
    mut piece_query: Query<(&mut Transform, &mut PieceComponent), With<PieceComponent>>,
    mut piece_stack: ResMut<PieceStack>,
) {
    let piece_count = piece_query.iter().len();
    let z_step = (MAX_PIECE_HEIGHT - MIN_PIECE_HEIGHT) / piece_count as f32;

    let mut stack_offset = 0;
    let mut i = 0;
    piece_stack.0.retain(|piece_entity| {
        let (mut transform, mut piece) = piece_query.get_mut(*piece_entity).unwrap();
        if piece.stack_pos == i {
            piece.stack_pos -= stack_offset;
            transform.translation.z = piece.stack_pos as f32 * z_step + MIN_PIECE_HEIGHT;
            i += 1;
            true
        } else {
            stack_offset += 1;
            i += 1;
            false
        }
    });
}
