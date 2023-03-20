use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use bevy::transform::components::Transform;
use image::RgbaImage;
use serde::{Deserialize, Serialize};

use crate::{Piece, PieceIndex, PieceMoveEvent, BORDER_SIZE_FRACTION};

const CONNECTION_DISTANCE: f32 = 30.0;
const CONNECTION_ANGLE: f32 = 0.5; // rad

#[derive(Serialize, Deserialize, bevy::ecs::system::Resource)]
pub struct Puzzle {
    image: crate::image::Image,
    puzzle_width: u8,
    puzzle_height: u8,
    piece_width: u32,
    piece_height: u32,
    piece_map: HashMap<PieceIndex, Arc<RwLock<Piece>>>,
    groups: Vec<Vec<Arc<RwLock<Piece>>>>,
}

impl Puzzle {
    pub fn new(image_path: &Path, target_piece_count: u16) -> Self {
        let file = std::fs::File::open(image_path).unwrap();
        let reader = std::io::BufReader::new(file);
        let mut image = image::io::Reader::new(reader)
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

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

        let mut border_size = piece_width.min(piece_height) / BORDER_SIZE_FRACTION;
        if border_size % 2 == 1 {
            border_size -= 1;
        }

        let mut image_w_border = RgbaImage::new(
            image.width() + 2 * border_size,
            image.height() + 2 * border_size,
        );

        for (x, y, pixel) in image.enumerate_pixels() {
            *(image_w_border.get_pixel_mut(x + border_size, y + border_size)) = *pixel;
        }

        image = image_w_border;

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
                let piece =
                    Piece::new(&puzzle, index, puzzle.groups.len(), &mut image, border_size);
                let piece_ref = Arc::new(RwLock::new(piece));

                puzzle.piece_map.insert(index, piece_ref.clone());
                puzzle.groups.push(vec![piece_ref]);
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

    #[allow(unused)]
    fn with_piece<T>(&self, index: &PieceIndex, op: impl FnOnce(&Piece) -> T) -> Option<T> {
        self.piece_map
            .get(index)
            .map(|piece_ref| op(&piece_ref.read().unwrap()))
    }

    #[allow(unused)]
    fn with_piece_mut<T>(
        &mut self,
        index: &PieceIndex,
        op: impl FnOnce(&mut Piece) -> T,
    ) -> Option<T> {
        self.piece_map
            .get_mut(index)
            .map(|piece_ref| op(&mut piece_ref.write().unwrap()))
    }

    pub fn with_pieces(&self, mut op: impl FnMut(&Piece)) {
        self.piece_map
            .values()
            .map(|piece_ref| op(&piece_ref.read().unwrap()))
            .collect()
    }

    #[allow(unused)]
    fn with_pieces_mut(&mut self, mut op: impl FnMut(&mut Piece)) {
        self.piece_map
            .values_mut()
            .map(|piece_ref| op(&mut piece_ref.write().unwrap()))
            .collect()
    }

    pub fn with_group(&self, group_index: usize, mut op: impl FnMut(&Piece)) {
        self.groups[group_index]
            .iter()
            .map(|piece_ref| op(&piece_ref.read().unwrap()))
            .collect()
    }

    pub fn with_group_mut(&mut self, group_index: usize, mut op: impl FnMut(&mut Piece)) {
        self.groups[group_index]
            .iter_mut()
            .map(|piece_ref| op(&mut piece_ref.write().unwrap()))
            .collect()
    }

    pub fn piece_width(&self) -> u32 {
        self.piece_width
    }

    pub fn piece_height(&self) -> u32 {
        self.piece_height
    }

    pub fn move_piece(&mut self, index: &PieceIndex, transform: Transform) -> Vec<PieceMoveEvent> {
        // TODO calculate delta and move whole group with it
        let piece_transform = self.with_piece(index, |piece| piece.transform).unwrap();
        let inverse_piece_transform =
            Transform::from_matrix(piece_transform.compute_matrix().inverse());
        let delta = transform.mul_transform(inverse_piece_transform);
        self.move_piece_rel(index, delta)
    }

    pub fn move_piece_rel(&mut self, index: &PieceIndex, delta: Transform) -> Vec<PieceMoveEvent> {
        let group_index = self.with_piece(index, |piece| piece.group_index).unwrap();
        let mut events = Vec::new();
        self.with_group_mut(group_index, |piece| {
            piece.transform = piece.transform.mul_transform(delta);
            events.push(PieceMoveEvent::from_piece(piece));
        });
        events
    }

    pub fn make_group_connections(&mut self, index: &PieceIndex) -> Vec<PieceMoveEvent> {
        let mut events = Vec::new();
        let mut piece_indices = Vec::new();
        let group_index = self.with_piece(index, |piece| piece.group_index).unwrap();
        self.with_group(group_index, |piece| piece_indices.push(piece.index()));
        for index in piece_indices {
            events.extend(self.make_piece_connections(&index));
        }
        events
    }

    fn make_piece_connections(&mut self, index: &PieceIndex) -> Vec<PieceMoveEvent> {
        let possible_neighbors = [
            (index.0.saturating_add(1), index.1),
            (index.0.saturating_sub(1), index.1),
            (index.0, index.1.saturating_add(1)),
            (index.0, index.1.saturating_sub(1)),
        ];

        let neighbors: Vec<_> = possible_neighbors
            .into_iter()
            .filter(|(row, col)| *row < self.puzzle_height && *col < self.puzzle_width)
            .map(|other| PieceIndex(other.0, other.1))
            .filter(|other| {
                other != index
                    && self.with_piece(index, |piece| piece.group_index)
                        != self.with_piece(&other, |piece| piece.group_index)
            })
            .collect();

        let mut connection_count = 0;
        let closest = neighbors
            .into_iter()
            .map(|other| self.single_connection_check(index, &other))
            .filter(|(_, distance, angle, _)| {
                *distance <= CONNECTION_DISTANCE && *angle <= CONNECTION_ANGLE
            })
            .inspect(|_| connection_count += 1)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(closest) = closest {
            let mut events = self.move_piece(index, closest.0);

            let new_group_index = self
                .with_piece(&closest.3, |piece| piece.group_index)
                .unwrap();
            let old_group_index = self
                .with_piece_mut(index, |piece| {
                    let old = piece.group_index;
                    old
                })
                .unwrap();
            self.with_group_mut(old_group_index, |piece| piece.group_index = new_group_index);
            let recruits = self.groups[old_group_index].drain(..).collect::<Vec<_>>();
            self.groups[new_group_index].extend(recruits);

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
    ) -> (Transform, f32, f32, PieceIndex) {
        let perfect = self
            .with_piece(other, |piece| {
                // todo account for sprite size
                let x = piece.transform.translation.x
                    + (index.1 as f32 - other.1 as f32) * self.piece_width as f32;
                let y = piece.transform.translation.y
                    + (other.0 as f32 - index.0 as f32) * self.piece_height as f32;
                Transform::from_xyz(x, y, 0.0)
            })
            .unwrap();

        let (distance, angle) = self
            .with_piece(index, |piece| {
                (
                    perfect
                        .translation
                        .truncate()
                        .distance(piece.transform.translation.truncate()), // TODO scale to piece size
                    perfect.rotation.angle_between(piece.transform.rotation),
                )
            })
            .unwrap();

        (perfect, distance, angle, other.clone())
    }
}
