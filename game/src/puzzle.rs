use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use bevy::transform::components::Transform;
use image::RgbaImage;
use serde::{Deserialize, Serialize};

use crate::{Piece, PieceIndex, PieceKind, PieceMoved};

pub const CONNECTION_DISTANCE_RATIO: f32 = 0.1;

#[derive(Serialize, Deserialize)]
struct Group {
    pieces: Vec<Arc<RwLock<Piece>>>,
    locked: bool,
}

#[derive(Serialize, Deserialize, bevy::ecs::system::Resource)]
pub struct Puzzle {
    image: crate::image::Image,
    puzzle_width: u8,
    puzzle_height: u8,
    piece_width: u32,
    piece_height: u32,
    piece_map: HashMap<PieceIndex, Arc<RwLock<Piece>>>,
    groups: Vec<Group>,
}

impl Debug for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Puzzle").finish()
    }
}

impl Puzzle {
    pub fn new(mut image: RgbaImage, target_piece_count: u16) -> Self {
        // compute puzzle width and height based while trying to make pieces as square as possible
        let image_ratio = f64::from(image.width()) / f64::from(image.height());
        let puzzle_height = (f64::from(target_piece_count) / image_ratio).sqrt();
        let puzzle_width = image_ratio * puzzle_height;

        let puzzle_height = puzzle_height.round().max(2.0) as u8;
        let puzzle_width = puzzle_width.round().max(2.0) as u8;

        // make sure piece sizes are even so tabs are centered.
        let mut piece_width = image.width() / u32::from(puzzle_width);
        if piece_width % 2 == 1 {
            piece_width -= 1;
        }

        let mut piece_height = image.height() / u32::from(puzzle_height);
        if piece_height % 2 == 1 {
            piece_height -= 1;
        }

        // crop pixels from right and bottom of image to make size multiple of piece size
        let mut image: image::RgbaImage = image::imageops::crop(
            &mut image,
            0,
            0,
            u32::from(puzzle_width) * piece_width,
            u32::from(puzzle_height) * piece_height,
        )
        .to_image();

        let piece_map = HashMap::new();
        let groups = Vec::new();

        let mut puzzle = Self {
            image: crate::image::Image::empty(),
            puzzle_width,
            puzzle_height,
            piece_width,
            piece_height,
            piece_map,
            groups,
        };

        for row in 0..puzzle_height {
            for col in 0..puzzle_width {
                let index = PieceIndex(row, col);
                let piece = Piece::new(&puzzle, index, puzzle.groups.len(), &mut image);
                let piece_ref = Arc::new(RwLock::new(piece));

                puzzle.piece_map.insert(index, piece_ref.clone());
                puzzle.groups.push(Group {
                    pieces: vec![piece_ref],
                    locked: false,
                });
            }
        }

        puzzle.image = image.into();
        puzzle
    }

    pub fn image(&self) -> crate::image::Image {
        self.image.clone()
    }

    pub fn puzzle_width(&self) -> u8 {
        self.puzzle_width
    }

    pub fn puzzle_height(&self) -> u8 {
        self.puzzle_height
    }

    pub fn with_piece<T>(&self, index: &PieceIndex, op: impl FnOnce(&Piece) -> T) -> Option<T> {
        self.piece_map
            .get(index)
            .map(|piece_ref| op(&piece_ref.read().unwrap()))
    }

    fn with_piece_mut<T>(
        &mut self,
        index: &PieceIndex,
        op: impl FnOnce(&mut Piece) -> T,
    ) -> Option<T> {
        self.piece_map
            .get_mut(index)
            .map(|piece_ref| op(&mut piece_ref.write().unwrap()))
    }

    pub fn with_pieces<T>(&self, mut op: impl FnMut(&Piece) -> T) -> Vec<T> {
        self.piece_map
            .values()
            .map(|piece_ref| op(&piece_ref.read().unwrap()))
            .collect()
    }

    #[allow(unused)]
    fn with_pieces_mut<T>(&mut self, mut op: impl FnMut(&mut Piece) -> T) -> Vec<T> {
        self.piece_map
            .values_mut()
            .map(|piece_ref| op(&mut piece_ref.write().unwrap()))
            .collect()
    }

    pub fn with_group<T>(
        &self,
        group_index: usize,
        mut op: impl FnMut(&Piece) -> T,
    ) -> Option<Vec<T>> {
        let group = self.groups.get(group_index);
        if let Some(group) = group {
            return Some(
                group
                    .pieces
                    .iter()
                    .map(|piece_ref| op(&piece_ref.read().unwrap()))
                    .collect(),
            );
        }
        None
    }

    pub fn with_group_mut<T>(
        &self,
        group_index: usize,
        mut op: impl FnMut(&mut Piece) -> T,
    ) -> Option<Vec<T>> {
        let group = self.groups.get(group_index);
        if let Some(group) = group {
            return Some(
                group
                    .pieces
                    .iter()
                    .map(|piece_ref| op(&mut piece_ref.write().unwrap()))
                    .collect(),
            );
        }
        None
    }

    pub fn piece_width(&self) -> u32 {
        self.piece_width
    }

    pub fn piece_height(&self) -> u32 {
        self.piece_height
    }

    pub fn move_piece(&mut self, index: &PieceIndex, transform: Transform) -> Vec<PieceMoved> {
        let piece_transform = self.with_piece(index, |piece| piece.transform).unwrap();
        let inverse_piece_transform =
            Transform::from_matrix(piece_transform.compute_matrix().inverse());
        let delta = transform.mul_transform(inverse_piece_transform);
        self.move_piece_rel(index, delta)
    }

    pub fn move_piece_rel(&mut self, index: &PieceIndex, delta: Transform) -> Vec<PieceMoved> {
        let group_index = self.with_piece(index, |piece| piece.group_index).unwrap();
        let mut events = Vec::new();
        if self.groups[group_index].locked {
                      return events;
        }
        self.with_group_mut(group_index, |piece| {
            piece.transform.translation += delta.translation;
            piece.transform.rotation *= delta.rotation;
            events.push(PieceMoved::from_piece(piece));
        });
        events
    }

    pub fn make_group_connections(&mut self, index: &PieceIndex) -> Vec<PieceMoved> {
        let mut events = Vec::new();
        let mut piece_indices = Vec::new();
        let group_index = self.with_piece(index, |piece| piece.group_index).unwrap();
        self.with_group(group_index, |piece| piece_indices.push(piece.index()));

        for index in &piece_indices {
            events.extend(self.make_piece_connections(index));
        }

        for index in piece_indices {
            events.extend(self.piece_lock_check(&index));
        }

        events
    }

    fn make_piece_connections(&mut self, index: &PieceIndex) -> Vec<PieceMoved> {
        let neighbors: Vec<_> = index
            .neighbors(self.puzzle_width, self.puzzle_height)
            .into_iter()
            .filter(|other| {
                self.with_piece(index, |piece| piece.group_index)
                    != self.with_piece(other, |piece| piece.group_index)
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
            let mut events = self.move_piece(index, closest.0);

            let new_group_index = self
                .with_piece(&closest.2, |piece| piece.group_index)
                .unwrap();
            let old_group_index = self
                .with_piece_mut(index, |piece| piece.group_index)
                .unwrap();
            self.with_group_mut(old_group_index, |piece| piece.group_index = new_group_index);
            let recruits = self.groups[old_group_index]
                .pieces
                .drain(..)
                .collect::<Vec<_>>();
            self.groups[new_group_index].pieces.extend(recruits);
            self.groups[new_group_index].locked =
                self.groups[new_group_index].locked || self.groups[old_group_index].locked;

            if connection_count > 1 {
                events.extend(self.make_piece_connections(index));
            }
            return events;
        }
        Vec::new()
    }

    fn single_connection_check(
        &mut self,
        index: &PieceIndex,
        other: &PieceIndex,
    ) -> (Transform, f32, PieceIndex) {
        let perfect = self
            .with_piece(other, |piece| {
                let x = piece.transform.translation.x
                    + (index.1 as f32 - other.1 as f32) * self.piece_width as f32;
                let y = piece.transform.translation.y
                    + (other.0 as f32 - index.0 as f32) * self.piece_height as f32;
                Transform::from_xyz(x, y, 0.0)
            })
            .unwrap();

        let distance = self
            .with_piece(index, |piece| {
                perfect
                    .translation
                    .truncate()
                    .distance(piece.transform.translation.truncate())
            })
            .unwrap();

        (perfect, distance, *other)
    }

    fn piece_lock_check(&mut self, index: &PieceIndex) -> Vec<PieceMoved> {
        use PieceKind::*;
        let kind = PieceKind::new(index, self.puzzle_width, self.puzzle_height);
        if matches!(kind,
            TopLeftCorner
            | TopRightCornerOdd
            | TopRightCornerEven
            | BottomLeftCornerOdd
            | BottomLeftCornerEven
            | BottomRightCornerOdd
            | BottomRightCornerEven) {
            let (piece_x, piece_y) = self
                .with_piece(index, |piece| {
                    (piece.transform.translation.x, piece.transform.translation.y)
                })
                .unwrap();
            let half_width = f32::from(self.puzzle_width) * self.piece_width as f32 / 2.0;
            let half_height = f32::from(self.puzzle_height) * self.piece_height as f32 / 2.0;

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

            let x_dist = (piece_x - target_x).abs();
            let y_dist = (piece_y - target_y).abs();
            let square_dist = x_dist * x_dist + y_dist * y_dist;
            let connection_dist =
                CONNECTION_DISTANCE_RATIO * self.piece_width.min(self.piece_height) as f32;
            if square_dist <= connection_dist * connection_dist {
                let events = self.move_piece_rel(
                    index,
                    Transform::from_xyz(target_x - piece_x, target_y - piece_y, 0.0),
                );
                self.lock_piece_group(index);
                return events;
            }
        }
        Vec::new()
    }

    fn lock_piece_group(&mut self, index: &PieceIndex) {
        let group_index = self.with_piece(index, |piece| piece.group_index).unwrap();
        self.groups[group_index].locked = true;
    }

    pub fn piece_group_locked(&self, index: &PieceIndex) -> bool {
        let group_index = self.with_piece(index, |piece| piece.group_index).unwrap();
        // true
        self.groups[group_index].locked
    }
}
