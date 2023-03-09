use image::Pixel;

use crate::piece;

fn main() -> anyhow::Result<()> {
    let file = std::fs::File::open("ymo.jpg")?;
    let reader = std::io::BufReader::new(file);
    let mut image = image::io::Reader::new(reader)
        .with_guessed_format()?
        .decode()?
        .to_rgba8();

    //let w = image.width();
    //let h = image.height();
    //let pw = w / 10;
    //let ph = h / 2;
    //let mut crop = image::imageops::crop(&mut image, w / 3, h / 3, pw, ph).to_image();

    //let svg = std::fs::read("middle_piece.svg").unwrap();
    //let mut options = usvg::Options::default();
    //options.shape_rendering = usvg::ShapeRendering::CrispEdges;

    //let tree = usvg::Tree::from_data(&svg, &options)?;
    //let sx = f64::from(pw) / tree.size.width();
    //let sy = f64::from(ph) / tree.size.height();
    //if let usvg::NodeKind::Group(root_group) = &mut *tree.root.borrow_mut() {
    //    root_group.transform.scale(sx, sy);
    //}

    //let mut pixmap = tiny_skia::Pixmap::new(pw, ph).unwrap();
    //resvg::render(
    //    &tree,
    //    usvg::FitTo::Original,
    //    tiny_skia::Transform::default(),
    //    pixmap.as_mut(),
    //);

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

    for (x, y, pixel) in crop.enumerate_pixels_mut() {
        pixel.channels_mut()[3] = pixmap.pixel(x, y).unwrap().alpha();
    }

    crop.save("crop.png").unwrap();
    Ok(())
}
