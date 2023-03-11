use anyhow::Result;

use game::*;

fn main() -> Result<()> {
    puzzle_test()?;
    Ok(())
}

fn puzzle_test() -> Result<()> {
    let file = std::fs::File::open("../ymo.jpg")?;
    let reader = std::io::BufReader::new(file);
    let mut image = image::io::Reader::new(reader)
        .with_guessed_format()?
        .decode()?
        .to_rgba8();
    let _puzzle = Puzzle::new(&mut image, 9);
    Ok(())
}
