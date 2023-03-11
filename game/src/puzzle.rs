use std::collections::{
    hash_map::{Values, ValuesMut},
    HashMap,
};

use serde::{Deserialize, Serialize};

use crate::piece::*;

#[derive(Serialize, Deserialize)]
pub struct Puzzle {
    image: Vec<u8>, // raw image::RgbaImage
    image_width: u32,
    image_height: u32,
    puzzle_width: u8,
    puzzle_height: u8,
    piece_map: HashMap<PieceIndex, Piece>,
    piece_width: u32,
    piece_height: u32,
}

impl Puzzle {
    pub fn new(image: &mut image::RgbaImage, target_piece_count: u16) -> Self {
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

        let image_width = image.width();
        let image_height = image.height();

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
                let piece = Piece::new(
                    index,
                    piece_width,
                    piece_height,
                    &mut image,
                    puzzle_width,
                    puzzle_height,
                );
                piece_map.insert(index, piece);
            }
        }

        Self {
            image: image.into_raw(),
            image_width,
            image_height,
            puzzle_width,
            puzzle_height,
            piece_map,
            piece_width,
            piece_height,
        }
    }

    pub fn image(self) -> image::RgbaImage {
        image::RgbaImage::from_raw(self.image_width, self.image_height, self.image).unwrap()
    }

    pub fn puzzle_width(&self) -> u8 {
        self.puzzle_width
    }

    pub fn puzzle_height(&self) -> u8 {
        self.puzzle_height
    }

    pub fn piece(&self, index: PieceIndex) -> Option<&Piece> {
        self.piece_map.get(&index)
    }

    pub fn piece_mut(&mut self, index: PieceIndex) -> Option<&mut Piece> {
        self.piece_map.get_mut(&index)
    }

    pub fn pieces(&self) -> Values<PieceIndex, Piece> {
        self.piece_map.values()
    }

    pub fn pieces_mut(&mut self) -> ValuesMut<PieceIndex, Piece> {
        self.piece_map.values_mut()
    }

    pub fn piece_width(&self) -> u32 {
        self.piece_width
    }

    pub fn piece_height(&self) -> u32 {
        self.piece_height
    }
}
