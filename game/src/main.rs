use image::Pixel;
use usvg::NodeExt;

fn main() -> anyhow::Result<()> {
    let file = std::fs::File::open("ymo.jpg")?;
    let reader = std::io::BufReader::new(file);
    let mut img = image::io::Reader::new(reader)
        .with_guessed_format()?
        .decode()?
        .to_rgba8();

    let w = img.width();
    let h = img.height();
    let pw = w / 10;
    let ph = h / 2;
    let mut crop = image::imageops::crop(&mut img, w / 3, h / 3, pw, ph).to_image();

    let svg = std::fs::read("middle_piece.svg").unwrap();
    let mut options = usvg::Options::default();
    options.shape_rendering = usvg::ShapeRendering::CrispEdges;

    let tree = usvg::Tree::from_data(&svg, &options)?;
    let sx = f64::from(pw) / tree.size.width();
    let sy = f64::from(ph) / tree.size.height();
    if let usvg::NodeKind::Group(root_group) = &mut *tree.root.borrow_mut() {
        root_group.transform.scale(sx, sy);
    }

    let mut pixmap = tiny_skia::Pixmap::new(pw, ph).unwrap();
    resvg::render(
        &tree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    );

    for (x, y, pixel) in crop.enumerate_pixels_mut() {
        pixel.channels_mut()[3] = pixmap.pixel(x, y).unwrap().alpha();
    }

    crop.save("crop.png").unwrap();
    Ok(())
}
