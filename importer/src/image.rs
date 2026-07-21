use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use fast_image_resize::{ResizeAlg, ResizeOptions, Resizer};
use image::{ImageBuffer, ImageReader, RgbaImage};
use std::io::Cursor;
#[must_use]
pub fn parse_bytes(bytes: &[u8]) -> Option<Image> {
    let image = parse_no_mips(bytes)?;
    let (width, height) = (image.width(), image.height());
    let (data, mips) = generate_mips_texture(image);
    Some(make_img(data, width, height, mips))
}
#[must_use]
fn parse_no_mips(bytes: &[u8]) -> Option<RgbaImage> {
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let rgba = image.to_rgba8();
    Some(rgba)
}
#[must_use]
fn make_img(rgba: Vec<u8>, width: u32, height: u32, mips: u32) -> Image {
    let mut image = Image::new_uninit(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(rgba);
    image.texture_descriptor.mip_level_count = mips;
    image
}
fn generate_mips_texture(image: RgbaImage) -> (Vec<u8>, u32) {
    let mip_count = calculate_mip_count(image.width(), image.height());
    let new_image_data = generate_mips(image, mip_count);
    (new_image_data, mip_count)
}
fn generate_mips(mut dyn_image: RgbaImage, mip_count: u32) -> Vec<u8> {
    let mut width = dyn_image.width();
    let mut height = dyn_image.height();
    let mut image_data = Vec::with_capacity(dyn_image.len() + dyn_image.len().div_ceil(3));
    image_data.extend(dyn_image.as_raw());
    let mut resizer = Resizer::new();
    let resize_alg = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(
            fast_image_resize::FilterType::Lanczos3,
        ))
        .use_alpha(false);
    for _ in 1..mip_count {
        width /= 2;
        height /= 2;
        let mut new: RgbaImage = ImageBuffer::from_raw(
            width,
            height,
            vec![0; usize::try_from(width * height * 4).unwrap()],
        )
        .unwrap();
        resizer.resize(&dyn_image, &mut new, &resize_alg).unwrap();
        image_data.extend(new.as_raw());
        dyn_image = new;
    }
    image_data
}
fn calculate_mip_count(width: u32, height: u32) -> u32 {
    (width.min(height) as f32).log2().floor() as u32 + 1
}
