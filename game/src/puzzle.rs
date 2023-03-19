use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use image::RgbaImage;
use serde::{Deserialize, Serialize};

use crate::{Piece, PieceIndex, PieceMoveEvent, BORDER_SIZE_FRACTION};

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

    pub fn piece_width(&self) -> u32 {
        self.piece_width
    }

    pub fn piece_height(&self) -> u32 {
        self.piece_height
    }

    pub fn move_piece(&mut self, index: &PieceIndex, x: f32, y: f32) -> Vec<PieceMoveEvent> {
        self.with_piece_mut(index, |piece| {
            let mut translation = &mut piece.transform.translation;
            translation.x = x;
            translation.y = y;
            vec![PieceMoveEvent::from_piece(piece)]
        })
        .unwrap()
    }

    pub fn move_piece_rel(&mut self, index: &PieceIndex, dx: f32, dy: f32) -> Vec<PieceMoveEvent> {
        self.with_piece_mut(index, |piece| {
            let mut translation = &mut piece.transform.translation;
            translation.x += dx;
            translation.y += dy;
            vec![PieceMoveEvent::from_piece(piece)]
        })
        .unwrap()
    }
}
