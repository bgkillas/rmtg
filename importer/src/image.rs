use bevy::asset::RenderAssetUsages;
use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use fast_image_resize::{ResizeAlg, ResizeOptions, Resizer};
use image::imageops::FilterType;
use image::{GenericImageView as _, ImageBuffer, ImageReader, RgbaImage};
use std::io::Cursor;
pub struct MipmapGeneratorSettings {
    pub anisotropic_filtering: u16,
    pub filter_type: FilterType,
    pub minimum_mip_resolution: u32,
    pub low_quality: bool,
}
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
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    )
}
#[must_use]
pub fn parse_bytes(bytes: &[u8]) -> Option<Image> {
    let mut image = parse_no_mips(bytes)?;
    generate_mips_texture(
        &mut image,
        &MipmapGeneratorSettings {
            anisotropic_filtering: 1,
            filter_type: FilterType::Lanczos3,
            minimum_mip_resolution: 64,
            low_quality: false,
        },
        &mut 0,
    )?;
    Some(image)
}
fn generate_mips_texture(
    image: &mut Image,
    settings: &MipmapGeneratorSettings,
    #[allow(unused)] added_cache_size: &mut usize,
) -> Option<()> {
    let mut dyn_image = try_into_dynamic(image.clone())?;
    let mip_count = calculate_mip_count(
        dyn_image.width(),
        dyn_image.height(),
        settings.minimum_mip_resolution,
        u32::MAX,
    );
    let new_image_data = generate_mips(&mut dyn_image, mip_count, settings);
    image.texture_descriptor.mip_level_count = mip_count;
    image.data = Some(new_image_data);
    Some(())
}
fn try_into_dynamic(image: Image) -> Option<RgbaImage> {
    let image_data = image.data?;
    ImageBuffer::from_raw(
        image.texture_descriptor.size.width,
        image.texture_descriptor.size.height,
        image_data,
    )
}
fn generate_mips(
    dyn_image: &mut RgbaImage,
    mip_count: u32,
    settings: &MipmapGeneratorSettings,
) -> Vec<u8> {
    let mut width = dyn_image.width();
    let mut height = dyn_image.height();
    let mut image_data = dyn_image.to_vec();
    let min = 1;
    let mut resizer = Resizer::new();
    let resize_alg = ResizeOptions::new()
        .resize_alg(match settings.filter_type {
            FilterType::Nearest => ResizeAlg::Nearest,
            FilterType::Triangle => ResizeAlg::Convolution(fast_image_resize::FilterType::Bilinear),
            FilterType::CatmullRom => {
                ResizeAlg::Convolution(fast_image_resize::FilterType::CatmullRom)
            }
            FilterType::Gaussian => ResizeAlg::Convolution(fast_image_resize::FilterType::Gaussian),
            FilterType::Lanczos3 => ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
        })
        .use_alpha(false);
    for _ in 0..mip_count {
        width /= 2;
        height /= 2;
        let mut new = ImageBuffer::from_raw(
            width,
            height,
            vec![0; usize::try_from(width * height * 4).unwrap()],
        )
        .unwrap();
        resizer.resize(dyn_image, &mut new, &resize_alg).unwrap();
        *dyn_image = new;
        image_data.append(&mut dyn_image.to_vec());
        if width <= min || height <= min {
            break;
        }
    }
    image_data
}
fn calculate_mip_count(
    mut width: u32,
    mut height: u32,
    minimum_mip_resolution: u32,
    max_mip_count: u32,
) -> u32 {
    let mut mip_level_count = 1;
    let min = 1;
    while width / 2 >= minimum_mip_resolution.max(min)
        && height / 2 >= minimum_mip_resolution.max(min)
        && mip_level_count < max_mip_count
    {
        width /= 2;
        height /= 2;
        mip_level_count += 1;
    }
    mip_level_count
}
