use image::Pixel;

fn main() -> anyhow::Result<()> {
    let file = std::fs::File::open("ymo.jpg")?;
    let reader = std::io::BufReader::new(file);
    let mut img = image::io::Reader::new(reader)
        .with_guessed_format()?
        .decode()?
        .to_rgba8();

    let w = img.width();
    let h = img.height();
    let mut crop = image::imageops::crop(&mut img, w / 3, h / 3, w / 2, h / 3).to_image();

    let svg = std::fs::read("middle_piece.svg").unwrap();
    let mut options = usvg::Options::default();
    options.shape_rendering = usvg::ShapeRendering::CrispEdges;
    let tree = usvg::Tree::from_data(&svg, &options)?;
    let mut pixmap = tiny_skia::Pixmap::new(w / 2, h / 3).unwrap();
    resvg::render(
        &tree,
        usvg::FitTo::Size(pixmap.width(), pixmap.height()),
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    );

    for (x, y, pixel) in crop.enumerate_pixels_mut() {
        pixel.channels_mut()[3] = pixmap.pixel(x, y).unwrap().alpha();
    }

    crop.save("crop.png").unwrap();
    Ok(())
}
