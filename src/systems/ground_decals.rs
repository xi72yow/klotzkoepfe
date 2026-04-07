use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::RngExt;

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
    /// Textur ist 1:1 zum Spielfeld, daher einfaches Offset-Mapping
    fn world_to_pixel(&self, world_pos: Vec2, field_w: f32, field_h: f32) -> (i32, i32) {
        let px = world_pos.x + field_w / 2.0;
        let py = -world_pos.y + field_h / 2.0;
        (px as i32, py as i32)
    }
}

fn create_decal_image(width: u32, height: u32) -> Image {
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
    image
}

/// Startup: Decal-Image und Layer-Sprite erstellen
pub fn setup_ground_decals(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    field: Res<crate::resources::GameField>,
) {
    // Textur = 1:1 zum Spielfeld (1 Pixel = 1 World Unit)
    let width = field.width as u32;
    let height = field.height as u32;

    let image = create_decal_image(width, height);
    let image_handle = images.add(image);

    commands.spawn((
        Sprite {
            image: image_handle.clone(),
            custom_size: Some(Vec2::new(field.width, field.height)),
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

/// Bei Feldgroessen-Aenderung: Textur vergroessern, alte Daten zentriert uebernehmen
pub fn resize_decal_texture(
    field: Res<crate::resources::GameField>,
    mut decal_map: ResMut<GroundDecalMap>,
    mut images: ResMut<Assets<Image>>,
    mut sprite_query: Query<&mut Sprite, With<GroundDecalLayer>>,
) {
    let new_w = field.width as u32;
    let new_h = field.height as u32;

    if new_w == decal_map.width && new_h == decal_map.height {
        return;
    }

    // Alte Pixeldaten rauskopieren
    let old_w = decal_map.width;
    let old_h = decal_map.height;
    let old_data: Option<Vec<u8>> = images.get(&decal_map.image_handle)
        .and_then(|img| img.data.as_ref().map(|d| d.clone()));

    // Neue Textur erstellen
    let new_image = create_decal_image(new_w, new_h);
    let new_handle = images.add(new_image);

    // Alte Daten zentriert in neue Textur kopieren
    if let Some(old_pixels) = old_data {
        if let Some(new_image) = images.get_mut(&new_handle) {
            if let Some(new_data) = new_image.data.as_mut() {
                let offset_x = (new_w as i32 - old_w as i32) / 2;
                let offset_y = (new_h as i32 - old_h as i32) / 2;

                for y in 0..old_h as i32 {
                    let dst_y = y + offset_y;
                    if dst_y < 0 || dst_y >= new_h as i32 { continue; }
                    for x in 0..old_w as i32 {
                        let dst_x = x + offset_x;
                        if dst_x < 0 || dst_x >= new_w as i32 { continue; }
                        let src_idx = (y as u32 * old_w + x as u32) as usize * 4;
                        let dst_idx = (dst_y as u32 * new_w + dst_x as u32) as usize * 4;
                        if src_idx + 3 < old_pixels.len() && dst_idx + 3 < new_data.len() {
                            new_data[dst_idx..dst_idx + 4].copy_from_slice(&old_pixels[src_idx..src_idx + 4]);
                        }
                    }
                }
            }
        }
    }

    // Alte Textur entfernen
    images.remove(&decal_map.image_handle);

    // Resource updaten
    decal_map.image_handle = new_handle.clone();
    decal_map.width = new_w;
    decal_map.height = new_h;

    // Sprite updaten
    for mut sprite in sprite_query.iter_mut() {
        sprite.image = new_handle.clone();
        sprite.custom_size = Some(Vec2::new(field.width, field.height));
    }
}

/// Verarbeitet alle pending Stamps und malt auf das Image
pub fn process_decal_stamps(
    mut decal_map: ResMut<GroundDecalMap>,
    mut images: ResMut<Assets<Image>>,
    field: Res<crate::resources::GameField>,
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
                let (cx, cy) = decal_map.world_to_pixel(position, field.width, field.height);
                stamp_circle(
                    data,
                    width,
                    height,
                    cx,
                    cy,
                    radius,
                    color,
                    0.7,
                );
            }
            DecalStamp::Burn { position, radius } => {
                let (cx, cy) = decal_map.world_to_pixel(position, field.width, field.height);
                let pixel_radius = radius;
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
                let (cx, cy) = decal_map.world_to_pixel(position, field.width, field.height);
                let pixel_radius = radius;
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

