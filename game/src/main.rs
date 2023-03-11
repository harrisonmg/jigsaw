use std::rc::Rc;

use anyhow::Result;
use image::Pixel;
use resvg::{tiny_skia, usvg};

use game::*;
use usvg::NodeExt;

fn main() -> Result<()> {
    puzzle_test()?;
    Ok(())
}

#[allow(dead_code)]
fn puzzle_test() -> Result<()> {
    let file = std::fs::File::open("nihon.jpg")?;
    let reader = std::io::BufReader::new(file);
    let mut image = image::io::Reader::new(reader)
        .with_guessed_format()?
        .decode()?
        .to_rgba8();
    let _puzzle = Puzzle::new(&mut image, 1000);
    Ok(())
}

#[allow(dead_code)]
fn cut_sprite_test() -> Result<()> {
    let file = std::fs::File::open("ymo.jpg")?;
    let reader = std::io::BufReader::new(file);
    let mut image = image::io::Reader::new(reader)
        .with_guessed_format()?
        .decode()?
        .to_rgba8();

    use PieceKind::*;
    for kind in [
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
    ] {
        let (piece_width, piece_height): (f64, f64) = (100.0, 100.0);
        let (tab_width, tab_height): (f64, f64) = (34.0, 34.0);
        let (north_tab, south_tab, east_tab, west_tab) = kind.tabs();
        let (north_blank, south_blank, east_blank, west_blank) = kind.blanks();

        let sprite_width = piece_width as u32 + tab_width as u32 * (east_tab + west_tab);
        let sprite_height = piece_height as u32 + tab_height as u32 * (north_tab + south_tab);

        let mut crop =
            image::imageops::crop(&mut image, 0, 0, sprite_width, sprite_height).to_image();

        let tree_size = usvg::Size::new(sprite_width.into(), sprite_height.into()).unwrap();
        let tree = usvg::Tree {
            size: tree_size,
            view_box: usvg::ViewBox {
                rect: tree_size.to_rect(0.0, 0.0),
                aspect: usvg::AspectRatio::default(),
            },
            root: usvg::Node::new(usvg::NodeKind::Group(usvg::Group::default())),
        };

        //

        let mut path_data = usvg::PathData::new();
        let mut cursor_x = west_tab as f64 * tab_width;
        let mut cursor_y = north_tab as f64 * tab_height;

        // start in northwest corner
        path_data.push_move_to(cursor_x, cursor_y);

        let mut rel_line = |dx: f64, dy: f64| {
            cursor_x += dx;
            cursor_y += dy;
            path_data.push_line_to(cursor_x, cursor_y);
        };

        let ns_tab_inner_size: f64 = 24.0;
        let ns_tab_outer_size: f64 = 38.0;
        let ns_corner_seg_size = (piece_width - ns_tab_inner_size) / 2.0;
        let ns_bulge_half_size = (ns_tab_outer_size - ns_tab_inner_size) / 2.0;

        let ew_tab_inner_size: f64 = 24.0;
        let ew_tab_outer_size: f64 = 38.0;
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

        //

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

        crop.save("crop.png").unwrap();

        // wait for enter press
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
    }

    Ok(())
}
