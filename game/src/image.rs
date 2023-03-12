use serde::{Deserialize, Serialize};

use bevy::render::texture::Image as BevyImageAsset;

#[derive(Serialize, Deserialize, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    raw: Vec<u8>,
}

impl Image {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl From<image::RgbaImage> for Image {
    fn from(value: image::RgbaImage) -> Self {
        Self {
            width: value.width(),
            height: value.height(),
            raw: value.into_raw(),
        }
    }
}

impl Into<BevyImageAsset> for Image {
    fn into(self) -> BevyImageAsset {
        BevyImageAsset::new(
            bevy::render::render_resource::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            self.raw,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        )
    }
}
