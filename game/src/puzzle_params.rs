use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PuzzleParams {
    pub puzzle_width: u8,
    pub puzzle_height: u8,
    pub piece_width: u32,
    pub piece_height: u32,
}
