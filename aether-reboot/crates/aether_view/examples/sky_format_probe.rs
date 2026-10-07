//! CPU-only probe for the production sky texture; no renderer or GPU is created.
use bevy::{
    asset::RenderAssetUsages,
    image::{
        CompressedImageFormats, Image, ImageAddressMode, ImageFilterMode, ImageSampler,
        ImageSamplerDescriptor, ImageType,
    },
    render::render_resource::{TextureFormat, TextureSampleType},
};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/textures/sky-world-linear.ktx2".to_owned());
    let bytes = std::fs::read(&path).expect("read original sky KTX2");
    let image = Image::from_buffer(
        &bytes,
        ImageType::Extension("ktx2"),
        CompressedImageFormats::NONE,
        false,
        ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::ClampToEdge,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            ..Default::default()
        }),
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("stock Bevy KTX2 loader accepts original linear sky");
    assert_eq!(image.texture_descriptor.format, TextureFormat::Rgba16Float);
    assert_eq!(image.texture_descriptor.size.width, 4096);
    assert_eq!(image.texture_descriptor.size.height, 2048);
    assert_eq!(image.texture_descriptor.mip_level_count, 1);
    assert_eq!(
        image.data.as_ref().expect("decoded pixels").len(),
        67_108_864
    );
    let payload_offset = usize::try_from(u64::from_le_bytes(
        bytes[80..88].try_into().expect("KTX2 level offset"),
    ))
    .expect("KTX2 offset fits platform");
    assert_eq!(
        image.data.as_ref().expect("decoded pixels").as_slice(),
        &bytes[payload_offset..],
        "stock loader must preserve every radiance sample and row orientation"
    );
    assert_eq!(image.asset_usage, RenderAssetUsages::RENDER_WORLD);
    assert_eq!(
        TextureFormat::Rgba16Float.sample_type(None, None),
        Some(TextureSampleType::Float { filterable: true })
    );
    assert_eq!(
        TextureFormat::Rgba32Float.sample_type(None, None),
        Some(TextureSampleType::Float { filterable: false })
    );
    println!(
        "PASS: {path}; stock Bevy RGBA16Float 4096x2048, 67108864 decoded/GPU bytes, filterable without optional features, Repeat U / ClampToEdge V, RENDER_WORLD only."
    );
}
