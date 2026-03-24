use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::Rng;

use crate::components::GroundDecalLayer;
use crate::constants::*;

/// Art des Decal-Stempels
#[derive(Clone)]
pub enum DecalStamp {
    Blood {
        position: Vec2,
        color: Color,
        radius: f32,
    },
    Burn {
        position: Vec2,
        radius: f32,
    },
    Ash {
        position: Vec2,
        radius: f32,
    },
}

/// Resource: haelt das Boden-Image und die pending Stamps
#[derive(Resource)]
pub struct GroundDecalMap {
    pub image_handle: Handle<Image>,
    pub width: u32,
    pub height: u32,
    pub dirty: bool,
    pub pending_stamps: Vec<DecalStamp>,
}

impl GroundDecalMap {
    /// Welt-Koordinaten (0,0 = Mitte) -> Pixel-Koordinaten (0,0 = oben-links)
    fn world_to_pixel(&self, world_pos: Vec2) -> (i32, i32) {
        let px = (world_pos.x + WINDOW_WIDTH / 2.0) * (self.width as f32 / WINDOW_WIDTH);
        let py = (-world_pos.y + WINDOW_HEIGHT / 2.0) * (self.height as f32 / WINDOW_HEIGHT);
        (px as i32, py as i32)
    }
}

/// Startup: Decal-Image und Layer-Sprite erstellen
pub fn setup_ground_decals(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let width = DECAL_TEXTURE_WIDTH;
    let height = DECAL_TEXTURE_HEIGHT;

    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();

    let image_handle = images.add(image);

    commands.spawn((
        Sprite {
            image: image_handle.clone(),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.5),
        GroundDecalLayer,
    ));

    commands.insert_resource(GroundDecalMap {
        image_handle,
        width,
        height,
        dirty: false,
        pending_stamps: Vec::new(),
    });
}

/// Verarbeitet alle pending Stamps und malt auf das Image
pub fn process_decal_stamps(
    mut decal_map: ResMut<GroundDecalMap>,
    mut images: ResMut<Assets<Image>>,
) {
    if decal_map.pending_stamps.is_empty() {
        return;
    }

    let stamps: Vec<DecalStamp> = decal_map.pending_stamps.drain(..).collect();
    let width = decal_map.width;
    let height = decal_map.height;

    let Some(image) = images.get_mut(&decal_map.image_handle) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };

    for stamp in stamps {
        match stamp {
            DecalStamp::Blood {
                position,
                color,
                radius,
            } => {
                let (cx, cy) = decal_map.world_to_pixel(position);
                stamp_circle(
                    data,
                    width,
                    height,
                    cx,
                    cy,
                    radius * (width as f32 / WINDOW_WIDTH),
                    color,
                    0.7,
                );
            }
            DecalStamp::Burn { position, radius } => {
                let (cx, cy) = decal_map.world_to_pixel(position);
                let pixel_radius = radius * (width as f32 / WINDOW_WIDTH);
                // Schwarzes Zentrum
                stamp_circle(
                    data,
                    width,
                    height,
                    cx,
                    cy,
                    pixel_radius * 0.6,
                    Color::srgb(0.05, 0.03, 0.02),
                    0.8,
                );
                // Brauner Rand
                stamp_ring(
                    data,
                    width,
                    height,
                    cx,
                    cy,
                    pixel_radius * 0.5,
                    pixel_radius,
                    Color::srgb(0.15, 0.1, 0.05),
                    0.5,
                );
            }
            DecalStamp::Ash { position, radius } => {
                let (cx, cy) = decal_map.world_to_pixel(position);
                let pixel_radius = radius * (width as f32 / WINDOW_WIDTH);
                let mut rng = rand::rng();
                // Mehrere kleine zufaellige Kleckse statt perfektem Kreis
                let blob_count = rng.random_range(4..7);
                for _ in 0..blob_count {
                    let ox = rng.random_range(-pixel_radius * 0.5..pixel_radius * 0.5) as i32;
                    let oy = rng.random_range(-pixel_radius * 0.5..pixel_radius * 0.5) as i32;
                    let blob_r = pixel_radius * rng.random_range(0.2..0.45);
                    let shade = rng.random_range(0.12..0.22);
                    stamp_circle(
                        data,
                        width,
                        height,
                        cx + ox,
                        cy + oy,
                        blob_r,
                        Color::srgb(shade, shade * 0.9, shade * 0.8),
                        rng.random_range(0.4..0.65),
                    );
                }
            }
        }
    }

    decal_map.dirty = true;
}

/// Gefuellter Kreis mit Alpha-Blending
fn stamp_circle(
    data: &mut [u8],
    width: u32,
    height: u32,
    cx: i32,
    cy: i32,
    radius: f32,
    color: Color,
    max_alpha: f32,
) {
    let srgba = color.to_srgba();
    let r_int = radius.ceil() as i32;

    for dy in -r_int..=r_int {
        for dx in -r_int..=r_int {
            let px = cx + dx;
            let py = cy + dy;
            if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > radius {
                continue;
            }

            let alpha = (1.0 - dist / radius) * max_alpha;
            let idx = (py as u32 * width + px as u32) as usize * 4;
            alpha_blend(data, idx, srgba.red, srgba.green, srgba.blue, alpha);
        }
    }
}

/// Ring (Donut) mit Alpha-Blending
fn stamp_ring(
    data: &mut [u8],
    width: u32,
    height: u32,
    cx: i32,
    cy: i32,
    inner_radius: f32,
    outer_radius: f32,
    color: Color,
    max_alpha: f32,
) {
    let srgba = color.to_srgba();
    let r_int = outer_radius.ceil() as i32;

    for dy in -r_int..=r_int {
        for dx in -r_int..=r_int {
            let px = cx + dx;
            let py = cy + dy;
            if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist < inner_radius || dist > outer_radius {
                continue;
            }

            let ring_width = outer_radius - inner_radius;
            let ring_pos = (dist - inner_radius) / ring_width;
            // Fade an den Raendern
            let edge_fade = (1.0 - (ring_pos * 2.0 - 1.0).abs()).min(1.0);
            let alpha = edge_fade * max_alpha;
            let idx = (py as u32 * width + px as u32) as usize * 4;
            alpha_blend(data, idx, srgba.red, srgba.green, srgba.blue, alpha);
        }
    }
}

/// Alpha-Blending eines Pixels
fn alpha_blend(data: &mut [u8], idx: usize, r: f32, g: f32, b: f32, new_a: f32) {
    if idx + 3 >= data.len() {
        return;
    }
    let old_r = data[idx] as f32 / 255.0;
    let old_g = data[idx + 1] as f32 / 255.0;
    let old_b = data[idx + 2] as f32 / 255.0;
    let old_a = data[idx + 3] as f32 / 255.0;

    let out_a = new_a + old_a * (1.0 - new_a);
    if out_a > 0.001 {
        data[idx] = ((r * new_a + old_r * old_a * (1.0 - new_a)) / out_a * 255.0) as u8;
        data[idx + 1] = ((g * new_a + old_g * old_a * (1.0 - new_a)) / out_a * 255.0) as u8;
        data[idx + 2] = ((b * new_a + old_b * old_a * (1.0 - new_a)) / out_a * 255.0) as u8;
        data[idx + 3] = (out_a * 255.0).min(255.0) as u8;
    }
}

/// Decal-Map komplett leeren (fuer Restart)
pub fn clear_decal_map(decal_map: &mut GroundDecalMap, images: &mut Assets<Image>) {
    if let Some(image) = images.get_mut(&decal_map.image_handle) {
        if let Some(data) = image.data.as_mut() {
            data.fill(0);
        }
    }
    decal_map.pending_stamps.clear();
    decal_map.dirty = false;
}
