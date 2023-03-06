use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum PieceKind {
    TopLeftCorner,

    TopRightCornerEven,
    TopRightOddCornerOdd,

    BottomLeftCornerEven,
    BottomLeftCornerOdd,

    BottomRightCornerEven,
    BottomRightCornerOdd,

    MiddleEven,
    MiddleOdd,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PieceIndex(u8, u8);

#[derive(Serialize, Deserialize)]
pub struct Piece {
    kind: PieceKind,
    index: PieceIndex,
    sprite: Vec<u8>, // raw RgbaImage from image crate
}

#[derive(Serialize, Deserialize)]
pub struct Puzzle {
    piece_width: u32,
    piece_height: u32,

    lobe_width: u32,
    lobe_height: u32,

    puzzle_width: u8,
    puzzle_height: u8,

    piece_map: HashMap<PieceIndex, Piece>,

    image: Vec<u8>, // raw RgbaImage from image crate
}

impl Puzzle {
    pub fn new(
        image: image::RgbaImage,
        lobe_to_piece_ratio: f64,
        target_piece_count: u16,
    ) -> anyhow::Result<Self> {
        // compute puzzle width and height based while trying to make pieces as square as possible
        let image_ratio: f64 = image.width().into() / image.height().into();
        let puzzle_height = (f64::from(target_piece_count) / image_ratio).sqrt();
        let puzzle_width = image_ratio * puzzle_height;

        let puzzle_height = puzzle_height.round().max(2.0) as u8;
        let puzzle_width = puzzle_width.round().max(2.0) as u8;

        let piece_map = HashMap::new();
    }
}
