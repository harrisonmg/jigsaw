use std::{
    collections::{
        hash_map::{Values, ValuesMut},
        HashMap,
    },
    path::Path,
};

use image::RgbaImage;
use serde::{Deserialize, Serialize};

use crate::{Piece, PieceIndex, PieceMoveEvent, BORDER_SIZE_FRACTION};

#[derive(Serialize, Deserialize, bevy::ecs::system::Resource)]
pub struct Puzzle {
    image: crate::image::Image,
    puzzle_width: u8,
    puzzle_height: u8,
    piece_map: HashMap<PieceIndex, Piece>,
    piece_width: u32,
    piece_height: u32,
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

        let mut piece_map = HashMap::new();
        for row in 0..puzzle_height {
            for col in 0..puzzle_width {
                let index = PieceIndex(row, col);
                let piece = Piece::new(
                    index,
                    piece_width,
                    piece_height,
                    border_size,
                    &mut image,
                    puzzle_width,
                    puzzle_height,
                );
                piece_map.insert(index, piece);
            }
        }

        Self {
            image: image.into(),
            puzzle_width,
            puzzle_height,
            piece_map,
            piece_width,
            piece_height,
        }
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

    pub fn piece(&self, index: PieceIndex) -> Option<&Piece> {
        self.piece_map.get(&index)
    }

    pub fn piece_mut(&mut self, index: &PieceIndex) -> Option<&mut Piece> {
        self.piece_map.get_mut(index)
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

    pub fn move_piece(&mut self, index: &PieceIndex, x: f32, y: f32) -> Vec<PieceMoveEvent> {
        let piece = self.piece_mut(index).unwrap();
        let mut translation = &mut piece.transform.translation;
        translation.x = x;
        translation.y = y;
        vec![PieceMoveEvent::from_piece(piece)]
    }

    pub fn move_piece_rel(&mut self, index: &PieceIndex, dx: f32, dy: f32) -> Vec<PieceMoveEvent> {
        let piece = self.piece_mut(index).unwrap();
        let mut translation = &mut piece.transform.translation;
        translation.x += dx;
        translation.y += dy;
        vec![PieceMoveEvent::from_piece(piece)]
    }
}
