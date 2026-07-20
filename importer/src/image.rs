use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_mod_mipmap_generator::{MipmapGeneratorSettings, generate_mips_texture};
use image::imageops::FilterType;
use image::{GenericImageView as _, ImageReader};
use std::io::Cursor;
#[must_use]
pub fn parse_no_mips(bytes: &[u8]) -> Option<Image> {
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    Some(make_img(rgba.into_raw(), width, height))
}
#[must_use]
pub fn make_img(rgba: Vec<u8>, width: u32, height: u32) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
#[must_use]
pub fn parse_bytes(bytes: &[u8]) -> Option<Image> {
    let mut image = parse_no_mips(bytes)?;
    generate_mips(&mut image)?;
    Some(image)
}
pub fn generate_mips(image: &mut Image) -> Option<()> {
    generate_mips_texture(
        image,
        &MipmapGeneratorSettings {
            anisotropic_filtering: 1,
            filter_type: FilterType::Lanczos3,
            minimum_mip_resolution: 64,
            ..MipmapGeneratorSettings::default()
        },
        &mut 0,
    )
    .ok()
}
