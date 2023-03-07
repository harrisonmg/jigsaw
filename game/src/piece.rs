use serde::{Deserialize, Serialize};

use crate::puzzle_params::PuzzleParams;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PieceIndex(pub u8, pub u8);

#[derive(Serialize, Deserialize)]
pub enum PieceKind {
    TopLeftCorner,

    TopRightCornerEven,
    TopRightCornerOdd,

    TopEdgeEven,
    TopEdgeOdd,

    BottomLeftCornerEven,
    BottomLeftCornerOdd,

    BottomEdgeEven,
    BottomEdgeOdd,

    BottomRightCornerEven,
    BottomRightCornerOdd,

    LeftEdgeEven,
    LeftEdgeOdd,

    RightEdgeEven,
    RightEdgeOdd,

    MiddleEven,
    MiddleOdd,
}

impl PieceKind {
    pub fn new(index: PieceIndex, params: &PuzzleParams) -> Self {
        use PieceKind::*;
        let PieceIndex(row, col) = index;
        let even = (row + col) % 2 == 0;

        if row == 0 {
            if col == 0 {
                TopLeftCorner
            } else if col == params.puzzle_width - 1 {
                if even {
                    TopRightCornerEven
                } else {
                    TopRightCornerOdd
                }
            } else {
                if even {
                    TopEdgeEven
                } else {
                    TopEdgeOdd
                }
            }
        } else if row == params.puzzle_height - 1 {
            if col == 0 {
                if even {
                    BottomLeftCornerEven
                } else {
                    BottomLeftCornerOdd
                }
            } else if col == params.puzzle_width - 1 {
                if even {
                    BottomRightCornerEven
                } else {
                    BottomRightCornerOdd
                }
            } else {
                if even {
                    BottomEdgeEven
                } else {
                    BottomEdgeOdd
                }
            }
        } else {
            if col == 0 {
                if even {
                    LeftEdgeEven
                } else {
                    LeftEdgeOdd
                }
            } else if col == params.puzzle_width - 1 {
                if even {
                    RightEdgeEven
                } else {
                    RightEdgeOdd
                }
            } else {
                if even {
                    MiddleEven
                } else {
                    MiddleOdd
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Piece {
    index: PieceIndex,
    kind: PieceKind,
    sprite_buf: Vec<u8>, // raw image::RgbaImage
    sprite_width: u32,
    sprite_height: u32,
}

impl Piece {
    pub fn new(index: PieceIndex, image: &mut image::RgbaImage, params: &PuzzleParams) -> Self {
        let kind = PieceKind::new(index, params);

        // TODO cut sprite, calculate pixel perfect lobes
        // probably have to create the SVG by hand
        let lobe_width = (LOBE_TO_PIECE_RATIO * f64::from(params.piece_width)) as u32;
        let lobe_height = (LOBE_TO_PIECE_RATIO * f64::from(params.piece_height)) as u32;

        let sprite = image::RgbaImage::new(params.piece_width, params.piece_height);

        Piece {
            index,
            kind,
            sprite_buf: sprite.into_raw()
            sprite_width: sprite.width(),
            sprite_height: sprite.height(),
        }
    }

    pub fn index(&self) -> PieceIndex {
        self.index
    }

    pub fn kind(&self) -> PieceKind {
        self.kind
    }

    pub fn sprite(&self) -> image::RgbaImage {
        image::RgbaImage::from_raw(self.sprite_width, self.sprite_height, self.sprite_buf).unwrap()
    }
}

const LOBE_TO_PIECE_RATIO: f64 = 1.0 / 3.0;
