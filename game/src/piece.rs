use std::rc::Rc;

use bevy::transform::components::Transform;
use image::Pixel;
use resvg::{tiny_skia, usvg};
use serde::{Deserialize, Serialize};
use usvg::NodeExt;

const TAB_LENGTH_RATIO: f64 = 0.34;
const TAB_OUTER_SIZE_RATIO: f64 = 0.38;
const TAB_INNER_SIZE_RATIO: f64 = 0.24;
pub(crate) const BORDER_SIZE_FRACTION: u32 = 10;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct PieceIndex(pub u8, pub u8);

impl PieceIndex {
    pub fn neighbors(self, puzzle_width: u8, puzzle_height: u8) -> Vec<Self> {
        let PieceIndex(row, col) = self;
        let mut neighbors = Vec::new();

        if row > 0 {
            neighbors.push(PieceIndex(row - 1, col))
        }

        if row < puzzle_height - 1 {
            neighbors.push(PieceIndex(row + 1, col))
        }

        if col > 0 {
            neighbors.push(PieceIndex(row, col - 1))
        }

        if col < puzzle_width - 1 {
            neighbors.push(PieceIndex(row, col + 1))
        }

        neighbors
    }
}

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

        #[allow(clippy::collapsible_else_if)]
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

    pub(crate) fn tabs(&self) -> (u32, u32, u32, u32) {
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

    pub(crate) fn blanks(&self) -> (u32, u32, u32, u32) {
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
    sprite: crate::image::Image,
    pub(crate) transform: Transform,
}

impl Piece {
    pub fn new(
        index: PieceIndex,
        piece_width: u32,
        piece_height: u32,
        border_size: u32,
        image: &mut image::RgbaImage,
        puzzle_width: u8,
        puzzle_height: u8,
    ) -> Self {
        let kind = PieceKind::new(index, puzzle_width, puzzle_height);

        let sprite = Piece::cut_sprite(index, piece_width, piece_height, border_size, image, kind);

        Piece {
            index,
            kind,
            sprite: sprite.into(),
            transform: Transform::IDENTITY,
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
        border_size: u32,
        image: &mut image::RgbaImage,
        kind: PieceKind,
    ) -> image::RgbaImage {
        let PieceIndex(row, col) = index;
        let (tab_width, tab_height) = Piece::tab_size(piece_width, piece_height);
        let (north_tab, south_tab, east_tab, west_tab) = kind.tabs();
        let (north_blank, south_blank, east_blank, west_blank) = kind.blanks();

        let sprite_width = piece_width + tab_width * (east_tab + west_tab) + 2 * border_size;
        let sprite_height = piece_height + tab_height * (north_tab + south_tab) + 2 * border_size;

        let mut crop = image::imageops::crop(
            image,
            col as u32 * piece_width - tab_width * west_tab,
            row as u32 * piece_height - tab_height * north_tab,
            sprite_width,
            sprite_height,
        )
        .to_image();

        let tree_size = usvg::Size::new(sprite_width.into(), sprite_height.into()).unwrap();
        let tree = usvg::Tree {
            size: tree_size,
            view_box: usvg::ViewBox {
                rect: tree_size.to_rect(0.0, 0.0),
                aspect: usvg::AspectRatio::default(),
            },
            root: usvg::Node::new(usvg::NodeKind::Group(usvg::Group::default())),
        };

        let mut path_data = usvg::PathData::new();
        let mut cursor_x: f64 = (west_tab * tab_width + border_size).into();
        let mut cursor_y: f64 = (north_tab * tab_height + border_size).into();

        // start in northwest corner
        path_data.push_move_to(cursor_x, cursor_y);

        let mut rel_line = |dx: f64, dy: f64| {
            cursor_x += dx;
            cursor_y += dy;
            path_data.push_line_to(cursor_x, cursor_y);
        };

        let piece_width: f64 = piece_width.into();
        let piece_height: f64 = piece_height.into();

        let tab_width: f64 = tab_width.into();
        let tab_height: f64 = tab_height.into();

        let mut ns_tab_inner_size: f64 = (TAB_INNER_SIZE_RATIO * piece_width).round();
        if ns_tab_inner_size / 2.0 != 0.0 {
            ns_tab_inner_size -= 1.0;
        }

        let mut ns_tab_outer_size: f64 = (TAB_OUTER_SIZE_RATIO * piece_width).round();
        if ns_tab_outer_size / 2.0 != 0.0 {
            ns_tab_outer_size -= 1.0;
        }

        let ns_corner_seg_size = (piece_width - ns_tab_inner_size) / 2.0;
        let ns_bulge_half_size = (ns_tab_outer_size - ns_tab_inner_size) / 2.0;

        let mut ew_tab_inner_size: f64 = (TAB_INNER_SIZE_RATIO * piece_height).round();
        if ew_tab_inner_size / 2.0 != 0.0 {
            ew_tab_inner_size -= 1.0;
        }

        let mut ew_tab_outer_size: f64 = (TAB_OUTER_SIZE_RATIO * piece_height).round();
        if ew_tab_outer_size / 2.0 != 0.0 {
            ew_tab_outer_size -= 1.0;
        }

        let ew_corner_seg_size = (piece_height - ew_tab_inner_size) / 2.0;
        let ew_bulge_half_size = (ew_tab_outer_size - ew_tab_inner_size) / 2.0;

        // northern eastward path
        rel_line(ns_corner_seg_size, 0.0);

        if north_tab > 0 {
            rel_line(-ns_bulge_half_size, -tab_height);
            rel_line(ns_tab_outer_size, 0.0);
            rel_line(-ns_bulge_half_size, tab_height);
        } else if north_blank > 0 {
            rel_line(-ns_bulge_half_size, tab_height);
            rel_line(ns_tab_outer_size, 0.0);
            rel_line(-ns_bulge_half_size, -tab_height);
        } else {
            rel_line(ns_tab_inner_size, 0.0);
        }

        rel_line(ns_corner_seg_size, 0.0);

        // eastern southward path
        rel_line(0.0, ew_corner_seg_size);

        if east_tab > 0 {
            rel_line(tab_width, -ew_bulge_half_size);
            rel_line(0.0, ew_tab_outer_size);
            rel_line(-tab_width, -ew_bulge_half_size);
        } else if east_blank > 0 {
            rel_line(-tab_width, -ew_bulge_half_size);
            rel_line(0.0, ew_tab_outer_size);
            rel_line(tab_width, -ew_bulge_half_size);
        } else {
            rel_line(0.0, ew_tab_inner_size);
        }

        rel_line(0.0, ew_corner_seg_size);

        // southern westward path
        rel_line(-ns_corner_seg_size, 0.0);

        if south_tab > 0 {
            rel_line(ns_bulge_half_size, tab_height);
            rel_line(-ns_tab_outer_size, 0.0);
            rel_line(ns_bulge_half_size, -tab_height);
        } else if south_blank > 0 {
            rel_line(ns_bulge_half_size, -tab_height);
            rel_line(-ns_tab_outer_size, 0.0);
            rel_line(ns_bulge_half_size, tab_height);
        } else {
            rel_line(-ns_tab_inner_size, 0.0);
        }

        rel_line(-ns_corner_seg_size, 0.0);

        // western northward path
        rel_line(0.0, -ew_corner_seg_size);

        if west_tab > 0 {
            rel_line(-tab_width, ew_bulge_half_size);
            rel_line(0.0, -ew_tab_outer_size);
            rel_line(tab_width, ew_bulge_half_size);
        } else if west_blank > 0 {
            rel_line(tab_width, ew_bulge_half_size);
            rel_line(0.0, -ew_tab_outer_size);
            rel_line(-tab_width, ew_bulge_half_size);
        } else {
            rel_line(0.0, -ew_tab_inner_size);
        }

        rel_line(0.0, -ew_corner_seg_size);

        tree.root.append_kind(usvg::NodeKind::Path(usvg::Path {
            fill: Some(usvg::Fill::default()), // black
            data: Rc::new(path_data),
            rendering_mode: usvg::ShapeRendering::CrispEdges,
            ..usvg::Path::default()
        }));

        let mut mask = resvg::tiny_skia::Pixmap::new(sprite_width, sprite_height).unwrap();
        resvg::render(
            &tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            mask.as_mut(),
        );

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

    pub fn sprite_clone(&self) -> crate::image::Image {
        self.sprite.clone()
    }

    pub fn transform(&self) -> Transform {
        self.transform
    }
}
