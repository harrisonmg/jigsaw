use std::fmt::Debug;

use anyhow::Result;
use bevy::{prelude::Vec3, transform::components::Transform, utils::HashMap};
use image::RgbaImage;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json_any_key::*;

use crate::{AnyGameEvent, Color, Piece, PieceIndex, PieceKind, PieceMovedEvent, Uuid};

pub const CONNECTION_DISTANCE_RATIO: f32 = 0.15;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Cursor {
    pub color: Color,
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize)]
struct Group {
    piece_indices: Vec<PieceIndex>,
    locked: bool,
}

#[derive(Serialize, Deserialize, bevy::ecs::system::Resource)]
pub struct Puzzle {
    image: crate::image::Image,
    num_cols: u8,
    num_rows: u8,
    piece_width: u32,
    piece_height: u32,

    #[serde(with = "any_key_map")]
    piece_map: HashMap<PieceIndex, Piece>,

    #[serde(with = "any_key_map")]
    held_pieces: HashMap<Uuid, PieceIndex>,

    groups: Vec<Group>,
}

impl Debug for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Puzzle")
            .field("num_cols", &self.num_cols)
            .field("num_rows", &self.num_rows)
            .field("piece_width", &self.piece_width)
            .field("piece_height", &self.piece_height)
            .finish()
    }
}

impl Puzzle {
    pub fn new(mut image: RgbaImage, target_piece_count: u16, randomize_position: bool) -> Self {
        // compute puzzle width and height based while trying to make pieces as square as possible
        let image_ratio = f64::from(image.width()) / f64::from(image.height());
        let num_rows = (f64::from(target_piece_count) / image_ratio).sqrt();
        let num_cols = image_ratio * num_rows;

        let num_rows = num_rows.round().max(2.0) as u8;
        let num_cols = num_cols.round().max(2.0) as u8;

        // make sure piece sizes are even so tabs are centered.
        let mut piece_width = image.width() / u32::from(num_cols);
        if piece_width % 2 == 1 {
            piece_width -= 1;
        }

        let mut piece_height = image.height() / u32::from(num_rows);
        if piece_height % 2 == 1 {
            piece_height -= 1;
        }

        // crop pixels from right and bottom of image to make size multiple of piece size
        let mut image: image::RgbaImage = image::imageops::crop(
            &mut image,
            0,
            0,
            u32::from(num_cols) * piece_width,
            u32::from(num_rows) * piece_height,
        )
        .to_image();

        let piece_map = HashMap::new();
        let held_pieces = HashMap::new();
        let groups = Vec::new();

        let mut puzzle = Self {
            image: crate::image::Image::empty(),
            num_cols,
            num_rows,
            piece_width,
            piece_height,
            piece_map,
            held_pieces,
            groups,
        };

        let mut rng = rand::thread_rng();
        let puzzle_width = puzzle.width() as f32;
        let puzzle_height = puzzle.height() as f32;
        let piece_big_side_len = piece_width.max(piece_height) as f32;
        let short_side_len = puzzle_width.min(puzzle_height);
        let long_side_len = puzzle_width.max(puzzle_height);

        for row in 0..num_rows {
            for col in 0..num_cols {
                let index = PieceIndex(row, col);
                let mut piece = Piece::new(&puzzle, index, puzzle.groups.len(), &mut image);

                if randomize_position {
                    let big_pos = (long_side_len + 2.0 * short_side_len) * (rng.gen::<f32>() - 0.5);
                    let mut small_pos = 3.0 * short_side_len * (rng.gen::<f32>() - 0.5);
                    if big_pos.abs() < long_side_len / 2.0 + piece_big_side_len
                        && small_pos.abs() < short_side_len / 2.0 + piece_big_side_len
                    {
                        small_pos =
                            (small_pos.abs() * 2.0 + short_side_len / 2.0 + piece_big_side_len)
                                * small_pos.signum();
                    }
                    piece.transform.translation = if puzzle_width >= puzzle_height {
                        Vec3::new(big_pos, small_pos, 0.0)
                    } else {
                        Vec3::new(small_pos, big_pos, 0.0)
                    };
                }

                puzzle.piece_map.insert(index, piece);
                puzzle.groups.push(Group {
                    piece_indices: vec![index],
                    locked: false,
                });
            }
        }

        puzzle.image = image.into();
        puzzle
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn deserialize(value: &str) -> Result<Self> {
        serde_json::from_str(value).map_err(anyhow::Error::from)
    }

    pub fn image(&self) -> crate::image::Image {
        self.image.clone()
    }

    pub fn num_cols(&self) -> u8 {
        self.num_cols
    }

    pub fn num_rows(&self) -> u8 {
        self.num_rows
    }

    pub fn piece(&self, index: &PieceIndex) -> Option<&Piece> {
        self.piece_map.get(index)
    }

    fn piece_mut(&mut self, index: &PieceIndex) -> Option<&mut Piece> {
        self.piece_map.get_mut(index)
    }

    pub fn with_pieces<T>(&self, op: impl FnMut(&Piece) -> T) -> Vec<T> {
        self.piece_map.values().map(op).collect()
    }

    pub fn with_group<T>(&self, group_index: usize, op: impl FnMut(&Piece) -> T) -> Option<Vec<T>> {
        self.groups.get(group_index).map(|group| {
            group
                .piece_indices
                .iter()
                .map(|index| self.piece(index).unwrap())
                .map(op)
                .collect()
        })
    }

    fn with_group_mut<T>(
        &mut self,
        group_index: usize,
        mut op: impl FnMut(&mut Piece) -> T,
    ) -> Vec<T> {
        let piece_indices = self.groups.get(group_index).unwrap().piece_indices.clone();
        piece_indices
            .iter()
            .map(|index| op(self.piece_mut(index).unwrap()))
            .collect()
    }

    pub fn piece_width(&self) -> u32 {
        self.piece_width
    }

    pub fn piece_height(&self) -> u32 {
        self.piece_height
    }

    pub fn width(&self) -> u32 {
        u32::from(self.num_cols) * self.piece_width
    }

    pub fn height(&self) -> u32 {
        u32::from(self.num_rows) * self.piece_height
    }

    pub fn try_move_piece(&mut self, index: &PieceIndex, x: f32, y: f32) -> Vec<PieceMovedEvent> {
        if self.piece_group_locked(index) {
            Vec::new()
        } else {
            self.move_piece(index, x, y)
        }
    }

    fn move_piece(&mut self, index: &PieceIndex, x: f32, y: f32) -> Vec<PieceMovedEvent> {
        let piece_transform = self.piece(index).unwrap().transform;
        let inverse_piece_transform =
            Transform::from_matrix(piece_transform.compute_matrix().inverse());

        let mut target_transform = Transform::from_xyz(x, y, piece_transform.translation.z);

        let clamp_half_size = self.width().min(self.height()) as f32 * 3.0;
        target_transform.translation = target_transform.translation.clamp(
            Vec3::new(-clamp_half_size, -clamp_half_size, f32::NEG_INFINITY),
            Vec3::new(clamp_half_size, clamp_half_size, f32::INFINITY),
        );

        let delta = target_transform.mul_transform(inverse_piece_transform);
        self.move_piece_rel(index, delta)
    }

    fn move_piece_rel(&mut self, index: &PieceIndex, delta: Transform) -> Vec<PieceMovedEvent> {
        let mut events = Vec::new();

        if delta == Transform::IDENTITY {
            return events;
        }

        let group_index = self.piece(index).unwrap().group_index;

        self.with_group_mut(group_index, |piece| {
            piece.transform.translation += delta.translation;
            piece.transform.rotation *= delta.rotation;
            events.push(PieceMovedEvent::from(&*piece));
        });
        events
    }

    pub fn make_group_connections(&mut self, index: &PieceIndex) -> Vec<PieceMovedEvent> {
        let mut events = Vec::new();
        let mut piece_indices = Vec::new();
        let group_index = self.piece(index).unwrap().group_index;
        self.with_group_mut(group_index, |piece| piece_indices.push(piece.index()));

        for index in &piece_indices {
            events.extend(self.make_piece_connections(index));
            events.extend(self.piece_lock_check(index));
        }

        events
    }

    fn make_piece_connections(&mut self, index: &PieceIndex) -> Vec<PieceMovedEvent> {
        let mut events = Vec::new();

        if self.piece_group_locked(index) {
            return events;
        }

        let neighbors: Vec<_> = index
            .neighbors(self.num_cols, self.num_rows)
            .into_iter()
            .filter(|other| {
                self.piece(index).unwrap().group_index != self.piece(other).unwrap().group_index
            })
            .collect();

        let mut connection_count = 0;
        let connection_dist =
            CONNECTION_DISTANCE_RATIO * self.piece_width.min(self.piece_height) as f32;
        let closest = neighbors
            .into_iter()
            .map(|other| self.single_connection_check(index, &other))
            .filter(|(_, distance, _)| *distance <= connection_dist)
            .inspect(|_| connection_count += 1)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(closest) = closest {
            let closest_x = closest.0.translation.x;
            let closest_y = closest.0.translation.y;
            events.extend(self.move_piece(index, closest_x, closest_y));

            let new_group_index = self.piece(&closest.2).unwrap().group_index;
            let old_group_index = self.piece(index).unwrap().group_index;
            self.with_group_mut(old_group_index, |piece| piece.group_index = new_group_index);
            let recruits = self.groups[old_group_index]
                .piece_indices
                .drain(..)
                .collect::<Vec<_>>();
            self.groups[new_group_index].piece_indices.extend(recruits);
            self.groups[new_group_index].locked =
                self.groups[new_group_index].locked || self.groups[old_group_index].locked;

            if connection_count > 1 {
                events.extend(self.make_piece_connections(index));
            }
        }
        events
    }

    fn single_connection_check(
        &mut self,
        index: &PieceIndex,
        other: &PieceIndex,
    ) -> (Transform, f32, PieceIndex) {
        let piece = self.piece(index).unwrap();
        let other_piece = self.piece(other).unwrap();

        let perfect = Transform::from_xyz(
            other_piece.transform.translation.x
                + (index.1 as f32 - other.1 as f32) * self.piece_width as f32,
            other_piece.transform.translation.y
                + (other.0 as f32 - index.0 as f32) * self.piece_height as f32,
            0.0,
        );

        let distance = perfect
            .translation
            .truncate()
            .distance(piece.transform.translation.truncate());

        (perfect, distance, *other)
    }

    fn piece_lock_check(&mut self, index: &PieceIndex) -> Vec<PieceMovedEvent> {
        use PieceKind::*;
        let kind = PieceKind::new(index, self.num_cols, self.num_rows);
        if matches!(
            kind,
            TopLeftCorner
                | TopRightCornerOdd
                | TopRightCornerEven
                | BottomLeftCornerOdd
                | BottomLeftCornerEven
                | BottomRightCornerOdd
                | BottomRightCornerEven
        ) {
            let translation = self.piece(index).unwrap().transform.translation;

            let half_width = f32::from(self.num_cols) * self.piece_width as f32 / 2.0;
            let half_height = f32::from(self.num_rows) * self.piece_height as f32 / 2.0;

            let half_piece_width = self.piece_width as f32 / 2.0;
            let half_piece_height = self.piece_height as f32 / 2.0;

            let target_x = match kind {
                TopLeftCorner | BottomLeftCornerOdd | BottomLeftCornerEven => {
                    -half_width + half_piece_width
                }
                TopRightCornerOdd
                | TopRightCornerEven
                | BottomRightCornerOdd
                | BottomRightCornerEven => half_width - half_piece_width,
                _ => 0.0,
            };

            let target_y = match kind {
                TopLeftCorner | TopRightCornerOdd | TopRightCornerEven => {
                    half_height - half_piece_height
                }
                BottomRightCornerOdd
                | BottomRightCornerEven
                | BottomLeftCornerOdd
                | BottomLeftCornerEven => -half_height + half_piece_height,
                _ => 0.0,
            };

            let x_dist = (translation.x - target_x).abs();
            let y_dist = (translation.y - target_y).abs();
            let square_dist = x_dist * x_dist + y_dist * y_dist;
            let connection_dist =
                CONNECTION_DISTANCE_RATIO * self.piece_width.min(self.piece_height) as f32;
            if square_dist <= connection_dist * connection_dist {
                let events = self.move_piece_rel(
                    index,
                    Transform::from_xyz(target_x - translation.x, target_y - translation.y, 0.0),
                );
                self.lock_piece_group(index);
                return events;
            }
        }
        Vec::new()
    }

    fn lock_piece_group(&mut self, index: &PieceIndex) {
        let group_index = self.piece(index).unwrap().group_index;
        self.groups[group_index].locked = true;
    }

    pub fn piece_group_locked(&self, index: &PieceIndex) -> bool {
        let group_index = self.piece(index).unwrap().group_index;
        self.groups[group_index].locked
    }

    pub fn piece_held(&self, index: &PieceIndex) -> bool {
        self.held_pieces
            .values()
            .any(|held_index| held_index == index)
    }

    pub fn can_pick_up(&self, index: &PieceIndex) -> bool {
        !self.piece_group_locked(index) && !self.piece_held(index)
    }

    pub fn apply_event(&mut self, event: AnyGameEvent) -> Vec<AnyGameEvent> {
        use AnyGameEvent::*;
        match event {
            PieceMoved(event) => self
                .try_move_piece(&event.index, event.x, event.y)
                .into_iter()
                .map(PieceMoved)
                .collect(),
            PiecePickedUp(event) => {
                if self.can_pick_up(&event.index) {
                    if let Some(player_id) = event.player_id {
                        self.held_pieces.insert(player_id, event.index);
                    }
                    vec![PiecePickedUp(event)]
                } else {
                    Vec::new()
                }
            }
            PiecePutDown(event) => {
                if let Some(player_id) = event.player_id {
                    if self
                        .held_pieces
                        .get(&player_id)
                        .map_or(false, |index| *index == event.index)
                    {
                        self.held_pieces.remove(&player_id);
                    }
                    return vec![PiecePutDown(event)];
                }
                Vec::new()
            }
            PieceConnection(event) => {
                let mut new_events: Vec<AnyGameEvent> = self
                    .make_group_connections(&event.index)
                    .into_iter()
                    .map(PieceMoved)
                    .collect();
                if !new_events.is_empty() {
                    new_events.push(PieceConnection(event));
                }
                new_events
            }
            PlayerCursorMoved(event) => {
                vec![PlayerCursorMoved(event)]
            }
            PlayerDisconnected(event) => {
                vec![PlayerDisconnected(event)]
            }
        }
    }
}
