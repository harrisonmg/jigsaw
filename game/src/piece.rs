use image::Pixel;
use serde::{Deserialize, Serialize};

const TAB_LENGTH_RATIO: f64 = 0.34;
const TAB_OUTER_SIZE_RATIO: f64 = 0.38;
const TAB_INNER_SIZE_RATIO: f64 = 0.24;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PieceIndex(pub u8, pub u8);

#[derive(Serialize, Deserialize, Clone, Copy)]
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
    pub fn new(index: PieceIndex, puzzle_width: u8, puzzle_height: u8) -> Self {
        use PieceKind::*;
        let PieceIndex(row, col) = index;
        let even = (row + col) % 2 == 0;

        if row == 0 {
            if col == 0 {
                TopLeftCorner
            } else if col == puzzle_width - 1 {
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
        } else if row == puzzle_height - 1 {
            if col == 0 {
                if even {
                    BottomLeftCornerEven
                } else {
                    BottomLeftCornerOdd
                }
            } else if col == puzzle_width - 1 {
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
            } else if col == puzzle_width - 1 {
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

    fn tabs(&self) -> (u32, u32, u32, u32) {
        use PieceKind::*;

        // north south east west
        match self {
            TopLeftCorner => (0, 0, 1, 0),

            TopRightCornerEven => (0, 0, 0, 1),
            TopRightCornerOdd => (0, 1, 0, 0),

            TopEdgeEven => (0, 0, 1, 1),
            TopEdgeOdd => (0, 1, 0, 0),

            BottomLeftCornerEven => (0, 0, 1, 0),
            BottomLeftCornerOdd => (1, 0, 0, 0),

            BottomEdgeEven => (0, 0, 1, 1),
            BottomEdgeOdd => (1, 0, 0, 0),

            BottomRightCornerEven => (0, 0, 0, 1),
            BottomRightCornerOdd => (1, 0, 0, 0),

            LeftEdgeEven => (0, 0, 1, 0),
            LeftEdgeOdd => (1, 1, 0, 0),

            RightEdgeEven => (0, 0, 0, 1),
            RightEdgeOdd => (1, 1, 0, 0),

            MiddleEven => (0, 0, 1, 1),
            MiddleOdd => (1, 1, 0, 0),
        }
    }

    fn blanks(&self) -> (u32, u32, u32, u32) {
        use PieceKind::*;

        // north south east west
        match self {
            TopLeftCorner => (0, 1, 0, 0),

            TopRightCornerEven => (0, 1, 0, 0),
            TopRightCornerOdd => (0, 0, 0, 1),

            TopEdgeEven => (0, 1, 0, 0),
            TopEdgeOdd => (0, 0, 1, 1),

            BottomLeftCornerEven => (1, 0, 0, 0),
            BottomLeftCornerOdd => (0, 0, 1, 0),

            BottomEdgeEven => (1, 0, 0, 0),
            BottomEdgeOdd => (0, 0, 1, 1),

            BottomRightCornerEven => (1, 0, 0, 0),
            BottomRightCornerOdd => (0, 0, 0, 1),

            LeftEdgeEven => (1, 1, 0, 0),
            LeftEdgeOdd => (0, 0, 1, 0),

            RightEdgeEven => (1, 1, 0, 0),
            RightEdgeOdd => (0, 0, 0, 1),

            MiddleEven => (1, 1, 0, 0),
            MiddleOdd => (0, 0, 1, 1),
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
    pub fn new(
        index: PieceIndex,
        piece_width: u32,
        piece_height: u32,
        image: &mut image::RgbaImage,
        puzzle_width: u8,
        puzzle_height: u8,
    ) -> Self {
        let kind = PieceKind::new(index, puzzle_width, puzzle_height);

        // TODO cut sprite, calculate pixel perfect tabs
        // probably have to create the SVG by hand

        let sprite = Piece::cut_sprite(index, piece_width, piece_height, image, kind);

        let sprite_width = sprite.width();
        let sprite_height = sprite.height();

        Piece {
            index,
            kind,
            sprite_buf: sprite.into_raw(),
            sprite_width,
            sprite_height,
        }
    }

    fn tab_size(piece_width: u32, piece_height: u32) -> (u32, u32) {
        (
            (TAB_LENGTH_RATIO * f64::from(piece_width)) as u32,
            (TAB_LENGTH_RATIO * f64::from(piece_height)) as u32,
        )
    }

    pub fn sprite_origin(&self, piece_width: u32, piece_height: u32) -> (u32, u32) {
        let (north_tab, _, _, west_tab) = self.kind.tabs();
        let (tab_width, tab_height) = Piece::tab_size(piece_width, piece_height);
        (tab_width * west_tab, tab_height * north_tab)
    }

    fn cut_sprite(
        index: PieceIndex,
        piece_width: u32,
        piece_height: u32,
        image: &mut image::RgbaImage,
        kind: PieceKind,
    ) -> image::RgbaImage {
        let PieceIndex(row, col) = index;
        let (tab_width, tab_height) = Piece::tab_size(piece_width, piece_height);
        let (north_tab, south_tab, east_tab, west_tab) = kind.tabs();
        let (north_blank, south_blank, east_blank, west_blank) = kind.blanks();

        let sprite_width = piece_width + tab_width * (east_tab + west_tab);
        let sprite_height = piece_height + tab_height * (north_tab + south_tab);

        let mut crop = image::imageops::crop(
            image,
            col as u32 * piece_width - tab_width * west_tab,
            row as u32 * piece_height - tab_height * north_tab,
            sprite_width,
            sprite_height,
        )
        .to_image();

        let mask = resvg::tiny_skia::Pixmap::new(sprite_width, sprite_height).unwrap();

        for (x, y, pixel) in crop.enumerate_pixels_mut() {
            pixel.channels_mut()[3] = mask.pixel(x, y).unwrap().alpha();
        }

        crop
    }

    pub fn index(&self) -> PieceIndex {
        self.index
    }

    pub fn kind(&self) -> PieceKind {
        self.kind
    }

    pub fn sprite(self) -> image::RgbaImage {
        image::RgbaImage::from_raw(self.sprite_width, self.sprite_height, self.sprite_buf).unwrap()
    }
}
