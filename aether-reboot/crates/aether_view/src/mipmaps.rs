//! Runtime texture filtering for the authored PNG/glTF surfaces. No source image
//! is rewritten. Work happens once when an uncompressed image enters Assets.
use bevy::{image::ImageFilterMode, prelude::*, render::render_resource::TextureFormat};
use std::collections::{HashMap, HashSet};

fn preserve_coverage(pixels: &mut [u8], cutoff: f32, target: f32) {
    let mut histogram = [0_u32; 256];
    for pixel in pixels.as_chunks::<4>().0 {
        histogram[pixel[3] as usize] += 1;
    }
    let count = (pixels.len() / 4) as f32;
    let reference = (cutoff * 255.0).ceil();
    let mut covered = 0;
    let mut best = (f32::MAX, 1.0_f32);
    // Select the alpha scale with the closest achievable coverage. Keep fully
    // transparent texels transparent; ties favour the smaller correction.
    for threshold in (1..=255).rev() {
        covered += histogram[threshold];
        let scale = reference / threshold as f32;
        let error = (covered as f32 / count - target).abs();
        if scale <= 8.0
            && (error < best.0 || (error == best.0 && (scale - 1.0).abs() < (best.1 - 1.0).abs()))
        {
            best = (error, scale);
        }
    }
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel[3] = (pixel[3] as f32 * best.1 + 0.0001)
            .floor()
            .clamp(0.0, 255.0) as u8;
    }
}

fn levels(
    pixels: Vec<u8>,
    mut width: usize,
    mut height: usize,
    srgb: bool,
    mask: Option<f32>,
) -> (Vec<u8>, u32) {
    let original_size = (width, height);
    let coverage = mask.map(|cutoff| {
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] as f32 / 255.0 >= cutoff)
            .count() as f32
            / (width * height) as f32
    });
    let linear: [f32; 256] = std::array::from_fn(|i| {
        let v = i as f32 / 255.0;
        if !srgb || v <= 0.04045 {
            if srgb { v / 12.92 } else { v }
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    });
    // One allocation contains every level. Cloning the 8K panorama and then
    // growing a second Vec doubled its CPU residency during browser startup.
    let mut total = pixels.len();
    let (mut w, mut h) = (width, height);
    while w > 1 || h > 1 {
        w = (w / 2).max(1);
        h = (h / 2).max(1);
        total += w * h * 4;
    }
    let mut all = pixels;
    all.reserve_exact(total - all.len());
    all.resize(total, 0);
    let mut offset = 0;
    let mut count = 1;
    while width > 1 || height > 1 {
        let nw = (width / 2).max(1);
        let nh = (height / 2).max(1);
        let end = offset + width * height * 4;
        let (previous, following) = all.split_at_mut(end);
        let pixels = &previous[offset..end];
        let next = &mut following[..nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                let (x0, x1) = (x * width / nw, (x + 1) * width / nw);
                let (y0, y1) = (y * height / nh, (y + 1) * height / nh);
                for channel in 0..4 {
                    let mut sum = 0.0;
                    let mut weight = 0.0;
                    for yy in y0..y1 {
                        for xx in x0..x1 {
                            let v = pixels[(yy * width + xx) * 4 + channel];
                            let w = if mask.is_some() && channel != 3 {
                                pixels[(yy * width + xx) * 4 + 3] as f32 / 255.0
                            } else {
                                1.0
                            };
                            weight += w;
                            sum += w * if channel == 3 {
                                v as f32 / 255.0
                            } else {
                                linear[v as usize]
                            };
                        }
                    }
                    let v = sum / weight.max(0.000001);
                    let encoded = if srgb && channel != 3 {
                        if v <= 0.0031308 {
                            v * 12.92
                        } else {
                            1.055 * v.powf(1.0 / 2.4) - 0.055
                        }
                    } else {
                        v
                    };
                    next[(y * nw + x) * 4 + channel] =
                        (encoded * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        offset = end;
        width = nw;
        height = nh;
        count += 1;
    }
    // Apply alpha correction only after constructing the entire unscaled
    // chain: propagating corrected alpha would compound it at each level.
    if let (Some(cutoff), Some(target)) = (mask, coverage) {
        let (mut w, mut h) = original_size;
        let mut start = w * h * 4;
        while w > 1 || h > 1 {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            let end = start + w * h * 4;
            // A single texel cannot represent fractional binary coverage;
            // preserve average alpha in the tail, where solid LOD takes over.
            if w >= 4 && h >= 4 {
                preserve_coverage(&mut all[start..end], cutoff, target);
            }
            start = end;
        }
    }
    (all, count)
}

#[derive(Default)]
pub struct MaskImages {
    cutoffs: HashMap<AssetId<Image>, f32>,
    pending: HashSet<AssetId<Image>>,
}
pub fn prepare(
    mut events: MessageReader<AssetEvent<Image>>,
    mut material_events: MessageReader<AssetEvent<StandardMaterial>>,
    materials: Res<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
    mut masks: Local<MaskImages>,
) {
    for event in material_events.read() {
        let (AssetEvent::Added { id } | AssetEvent::Modified { id }) = event else {
            continue;
        };
        if let Some(material) = materials.get(*id)
            && let AlphaMode::Mask(cutoff) = material.alpha_mode
            && cutoff > 0.0
            && cutoff < 1.0
            && let Some(texture) = &material.base_color_texture
            && masks.cutoffs.insert(texture.id(), cutoff) != Some(cutoff)
        {
            masks.pending.insert(texture.id());
        }
    }
    for event in events.read() {
        if let AssetEvent::Removed { id } = event {
            masks.cutoffs.remove(id);
            masks.pending.remove(id);
            continue;
        }
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } = event
            && images
                .get(*id)
                .is_some_and(|image| image.texture_descriptor.mip_level_count == 1)
        {
            masks.pending.insert(*id);
        }
    }
    let pending: Vec<_> = masks.pending.iter().copied().collect();
    for id in pending {
        // Dynamic glyph atlases and render targets have no source asset path.
        // Their owners can update level zero later, so never generate stale mips.
        if assets.get_path(id).is_none() {
            masks.pending.remove(&id);
            continue;
        }
        let Some(mut image) = images.get_mut(id) else {
            continue;
        };
        masks.pending.remove(&id);
        let desc = &image.texture_descriptor;
        if desc.size.depth_or_array_layers != 1 || desc.size.width < 64 || desc.size.height < 64 {
            continue;
        }
        let srgb = match desc.format {
            TextureFormat::Rgba8UnormSrgb => true,
            TextureFormat::Rgba8Unorm => false,
            _ => continue,
        };
        let (w, h) = (desc.size.width as usize, desc.size.height as usize);
        let Some(mut data) = image.data.take() else {
            continue;
        };
        if data.len() < w * h * 4 {
            image.data = Some(data);
            continue;
        }
        data.truncate(w * h * 4);
        let (data, count) = levels(data, w, h, srgb, masks.cutoffs.get(&id).copied());
        image.data = Some(data);
        image.texture_descriptor.mip_level_count = count;
        let sampler = image.sampler.get_or_init_descriptor();
        sampler.min_filter = ImageFilterMode::Linear;
        sampler.mag_filter = ImageFilterMode::Linear;
        sampler.mipmap_filter = ImageFilterMode::Linear;
        sampler.anisotropy_clamp = 8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dynamic_glyph_atlas_keeps_its_single_writable_level() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<StandardMaterial>()
            .add_systems(Update, prepare);
        let image = Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &[255, 255, 255, 127],
            TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::MAIN_WORLD
                | bevy::asset::RenderAssetUsages::RENDER_WORLD,
        );
        let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        app.update();
        app.update();
        let images = app.world().resource::<Assets<Image>>();
        let atlas = images.get(&handle).unwrap();
        assert_eq!(atlas.texture_descriptor.mip_level_count, 1);
        assert_eq!(atlas.data.as_ref().unwrap().len(), 64 * 64 * 4);
    }
    #[test]
    fn mipmap_averages_light_in_linear_space() {
        let mut pixels = vec![0, 0, 0, 255];
        pixels.extend([255, 255, 255, 255].repeat(3));
        let (data, count) = levels(pixels, 2, 2, true, None);
        assert_eq!(count, 2);
        assert_eq!(data.len(), 20);
        assert!((data[16] as i32 - 225).abs() <= 1);
        assert_eq!(data[19], 255);
    }
    #[test]
    fn non_power_of_two_mipmap_includes_last_row_and_column() {
        let pixels = [72, 91, 114, 255].repeat(3 * 5);
        let (data, count) = levels(pixels, 3, 5, true, None);
        assert_eq!(count, 3);
        assert_eq!(data.len(), (3 * 5 + 2 + 1) * 4);
        assert_eq!(&data[data.len() - 4..], &[72, 91, 114, 255]);
    }
    #[test]
    fn masked_leaf_color_does_not_average_with_transparent_black() {
        let mut pixels = [0, 0, 0, 0].repeat(4);
        pixels[..4].copy_from_slice(&[255, 32, 0, 255]);
        let (data, _) = levels(pixels, 2, 2, true, Some(0.33));
        assert_eq!(&data[16..], &[255, 32, 0, 64]);
    }
    #[test]
    fn narrow_leaf_coverage_survives_minification_without_changing_source() {
        let mut pixels = Vec::new();
        for y in 0..64 {
            for x in 0..64 {
                // Several narrow leaf tips of different projected areas.
                // Uniform subpixel noise would contain no recoverable spatial
                // silhouette, so no single alpha scale can preserve that case.
                let leaf_area = ((x / 8) * 7 + (y / 8) * 11) % 17;
                let alpha = if (y % 8) * 8 + x % 8 < leaf_area {
                    255
                } else {
                    0
                };
                pixels.extend([60, 130, 40, alpha]);
            }
        }
        let coverage = |data: &[u8]| {
            data.as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] >= 85)
                .count() as f32
                / (data.len() / 4) as f32
        };
        let target = coverage(&pixels);
        let (ordinary, _) = levels(pixels.clone(), 64, 64, true, None);
        let (masked, _) = levels(pixels.clone(), 64, 64, true, Some(0.33));
        assert_eq!(&masked[..pixels.len()], &pixels);
        let start = (64 * 64 + 32 * 32 + 16 * 16) * 4;
        let end = start + 8 * 8 * 4;
        assert_eq!(coverage(&ordinary[start..end]), 0.0);
        assert!(
            (coverage(&masked[start..end]) - target).abs() < 0.055,
            "source {target}, minified {}",
            coverage(&masked[start..end])
        );
    }
}
