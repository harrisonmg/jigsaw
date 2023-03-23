use game::*;

#[allow(unused)]
fn main() {
    let file = std::fs::File::open("ymo.jpg").unwrap();
    let reader = std::io::BufReader::new(file);
    let image = image::io::Reader::new(reader)
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
    let puzzle = Puzzle::new(image, 9);
}
