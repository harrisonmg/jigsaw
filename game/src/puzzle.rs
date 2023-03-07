use std::collections::{
    hash_map::{Values, ValuesMut},
    HashMap,
};

use serde::{Deserialize, Serialize};

use crate::piece::*;
use crate::puzzle_params::PuzzleParams;

#[derive(Serialize, Deserialize)]
pub struct Puzzle {
    params: PuzzleParams,
    image: Vec<u8>, // raw image::RgbaImage
    piece_map: HashMap<PieceIndex, Piece>,
}

impl Puzzle {
    pub fn new(image: &mut image::RgbaImage, target_piece_count: u16) -> Self {
        // compute puzzle width and height based while trying to make pieces as square as possible
        let image_ratio = f64::from(image.width()) / f64::from(image.height());
        let puzzle_height = (f64::from(target_piece_count) / image_ratio).sqrt();
        let puzzle_width = image_ratio * puzzle_height;

        let puzzle_height = puzzle_height.round().max(2.0) as u8;
        let puzzle_width = puzzle_width.round().max(2.0) as u8;

        let piece_width = image.width() / u32::from(puzzle_width);
        let piece_height = image.height() / u32::from(puzzle_height);

        let params = PuzzleParams {
            puzzle_width,
            puzzle_height,
            piece_width,
            piece_height,
        };

        // crop pixels from right and bottom of image to make size multiple of piece size
        let mut image: image::RgbaImage = image::imageops::crop(
            image,
            0,
            0,
            u32::from(puzzle_width) * piece_width,
            u32::from(puzzle_height) * piece_height,
        )
        .to_image();

        let mut piece_map = HashMap::new();
        for row in 0..puzzle_height {
            for col in 0..puzzle_width {
                let index = PieceIndex(row, col);
                let piece = Piece::new(index, &mut image, &params);
                piece_map.insert(index, piece);
            }
        }

        Self {
            params,
            image: image.into_raw(),
            piece_map,
        }
    }

    pub fn puzzle_width(&self) -> u8 {
        self.params.puzzle_width
    }

    pub fn puzzle_height(&self) -> u8 {
        self.params.puzzle_height
    }

    pub fn piece_width(&self) -> u32 {
        self.params.piece_width
    }

    pub fn piece_height(&self) -> u32 {
        self.params.piece_height
    }

    pub fn piece(&self, index: PieceIndex) -> Option<&Piece> {
        self.piece_map.get(&index)
    }

    pub fn piece_mut(&mut self, index: PieceIndex) -> Option<&mut Piece> {
        self.piece_map.get_mut(&index)
    }

    pub fn pieces<'a>(&'a self) -> Values<'a, PieceIndex, Piece> {
        self.piece_map.values()
    }

    pub fn pieces_mut<'a>(&'a mut self) -> ValuesMut<'a, PieceIndex, Piece> {
        self.piece_map.values_mut()
    }
}
