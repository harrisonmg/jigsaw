use std::collections::VecDeque;

use bevy::{
    prelude::*, render::mesh::VertexAttributeValues, sprite::MaterialMesh2dBundle, utils::HashMap,
};

use game::{PieceIndex, PieceKind, PieceMovedEvent, Puzzle};

use crate::{
    better_quad::BetterQuad, material::PieceMaterial, states::AppState, ui::LoadingMessage,
};

pub const MIN_PIECE_HEIGHT: f32 = 500.0;
pub const MAX_PIECE_HEIGHT: f32 = 900.0;

pub struct PiecePlugin;

impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Cutting), cutting_setup)
            .add_systems(Update, cut_pieces.run_if(in_state(AppState::Cutting)))
            .add_systems(
                Update,
                (move_piece, sort_pieces).run_if(in_state(AppState::Playing)),
            );
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

struct CachedPieceShape {
    mask_handle: Handle<Image>,
    shadow_handle: Handle<Image>,
    sprite_size: Vec2,
    sprite_origin: Vec2,
    shadow_x_offset: f32,
    shadow_y_offset: f32,
}

#[derive(Resource)]
struct PieceShapeCache(HashMap<PieceKind, CachedPieceShape>);

#[derive(Bundle)]
pub struct PieceBundle {
    piece: PieceComponent,
    mesh_bundle: MaterialMesh2dBundle<PieceMaterial>,
}

impl PieceBundle {
    fn new(
        index: PieceIndex,
        translation: Vec3,
        cached_shape: &CachedPieceShape,
        puzzle_texture: Handle<Image>,
        crop_x: u32,
        crop_y: u32,
        full_width: u32,
        full_height: u32,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<PieceMaterial>,
    ) -> Self {
        let sprite_size = cached_shape.sprite_size;
        let sprite_origin = cached_shape.sprite_origin;

        let piece_component = PieceComponent {
            index,
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

        // Encode UV rect as vertex colors so the fragment shader can remap UVs
        // into the full puzzle image. xy = offset, zw = scale.
        let uv_color: [f32; 4] = [
            crop_x as f32 / full_width as f32,
            crop_y as f32 / full_height as f32,
            sprite_size.x / full_width as f32,
            sprite_size.y / full_height as f32,
        ];
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![uv_color; 4]);

        let mesh_handle = meshes.add(mesh);

        let material = materials.add(PieceMaterial {
            puzzle_texture,
            mask_texture: cached_shape.mask_handle.clone(),
        });

        let mut translation = translation;
        translation.z = MIN_PIECE_HEIGHT;
        let mesh_bundle = MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material,
            transform: Transform::from_translation(translation),
            ..Default::default()
        };

        Self {
            piece: piece_component,
            mesh_bundle,
        }
    }
}

#[derive(Resource)]
pub struct PieceMap(pub HashMap<PieceIndex, Entity>);

#[derive(Resource)]
pub struct PieceStack(pub VecDeque<Entity>);

#[derive(Resource)]
struct PuzzleTexture(pub Handle<Image>);

#[derive(Resource)]
struct CurrentPieceToCut(pub u32);

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

fn cutting_setup(
    mut commands: Commands,
    piece_query: Query<Entity, With<PieceComponent>>,
    puzzle: Res<Puzzle>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    commands.insert_resource(PieceMap(HashMap::new()));
    commands.insert_resource(PieceStack(VecDeque::new()));
    commands.insert_resource(CurrentPieceToCut(0));

    let rgba_image = puzzle.rgba_image();
    let game_image: game::image::Image = rgba_image.into();
    let bevy_image: Image = game_image.into();
    let texture_handle = image_assets.add(bevy_image);
    commands.insert_resource(PuzzleTexture(texture_handle));

    // Pre-compute mask and shadow textures for all 17 piece kinds
    let mut shape_cache = HashMap::new();
    for kind in PieceKind::ALL {
        let (mask_sprite, shadow_sprite) =
            kind.render_mask_and_shadow(puzzle.piece_width(), puzzle.piece_height());

        let sprite_size = Vec2::new(
            mask_sprite.image.width() as f32,
            mask_sprite.image.height() as f32,
        );
        let sprite_origin = Vec2::new(mask_sprite.origin_x as f32, mask_sprite.origin_y as f32);
        let shadow_x_offset =
            shadow_sprite.image.width() as f32 / 2.0 - shadow_sprite.origin_x as f32;
        let shadow_y_offset =
            shadow_sprite.image.height() as f32 / 2.0 - shadow_sprite.origin_y as f32;

        let mask_handle = image_assets.add(mask_sprite.image.into());
        let shadow_handle = image_assets.add(shadow_sprite.image.into());

        shape_cache.insert(
            kind,
            CachedPieceShape {
                mask_handle,
                shadow_handle,
                sprite_size,
                sprite_origin,
                shadow_x_offset,
                shadow_y_offset,
            },
        );
    }
    commands.insert_resource(PieceShapeCache(shape_cache));

    for piece_entity in piece_query.iter() {
        commands
            .get_entity(piece_entity)
            .unwrap()
            .despawn_recursive();
    }
}

const PIECES_PER_FRAME: u32 = 50;

#[allow(clippy::too_many_arguments)]
fn cut_pieces(
    puzzle_texture: Res<PuzzleTexture>,
    shape_cache: Res<PieceShapeCache>,
    mut current_piece: ResMut<CurrentPieceToCut>,
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PieceMaterial>>,
    mut loading_msg: ResMut<LoadingMessage>,
    mut piece_map: ResMut<PieceMap>,
    mut piece_stack: ResMut<PieceStack>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    let batch_end = (current_piece.0 + PIECES_PER_FRAME).min(puzzle.piece_count());

    loading_msg.0 = format!(
        "Cutting pieces ( {} / {} )",
        batch_end,
        puzzle.piece_count()
    );

    while current_piece.0 < batch_end {
        let index = PieceIndex(
            current_piece.0 / puzzle.num_cols(),
            current_piece.0 % puzzle.num_cols(),
        );

        let piece = puzzle.piece(&index).unwrap();
        let cached_shape = shape_cache.0.get(&piece.kind()).unwrap();
        let (crop_x, crop_y) = piece.crop_offset(puzzle.as_ref());

        let piece_bundle = PieceBundle::new(
            index,
            piece.translation(),
            cached_shape,
            puzzle_texture.0.clone(),
            crop_x,
            crop_y,
            puzzle.width(),
            puzzle.height(),
            &mut meshes,
            &mut materials,
        );
        let piece_entity = commands.spawn(piece_bundle).id();

        let shadow = SpriteBundle {
            transform: Transform::from_xyz(
                cached_shape.shadow_x_offset,
                cached_shape.shadow_y_offset,
                -MIN_PIECE_HEIGHT,
            ),
            texture: cached_shape.shadow_handle.clone(),
            ..Default::default()
        };
        let shadow_entity = commands.spawn(shadow).id();

        commands
            .entity(piece_entity)
            .push_children(&[shadow_entity]);

        piece_map.0.insert(index, piece_entity);

        if puzzle.piece_group_locked(&index) {
            piece_stack.0.push_back(piece_entity);
        } else {
            piece_stack.0.push_front(piece_entity);
        }

        current_piece.0 += 1;
    }

    if current_piece.0 >= puzzle.piece_count() {
        next_state.set(AppState::Playing);
    }
}

fn move_piece(
    mut piece_moved_events: EventReader<PieceMovedEvent>,
    mut piece_query: Query<&mut Transform, With<PieceComponent>>,
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
