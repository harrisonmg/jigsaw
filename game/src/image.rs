use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    raw: Vec<u8>, // image::RgbaImage::into_raw()
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

impl Into<bevy::render::texture::Image> for Image {
    fn into(self) -> bevy::render::texture::Image {
        bevy::render::texture::Image::new(
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
