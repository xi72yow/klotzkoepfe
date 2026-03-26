use bevy::prelude::*;
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_graph::{InternedRenderLabel, InternedRenderSubGraph, RenderLabel, RenderSubGraph};
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

use crate::constants::*;
use crate::resources::GameSettings;

/// Marker fuer die Hauptkamera
#[derive(Component)]
pub struct MainCamera;

/// Post-Processing Pixelation Material
/// Wird als Component auf die Kamera gelegt
#[derive(Component, Clone, Copy, ShaderType)]
pub struct PixelationMaterial {
    pub pixel_size: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub scanline_intensity: f32,
    pub chromatic_aberration: f32,
    pub vignette_intensity: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for PixelationMaterial {
    fn default() -> Self {
        Self {
            pixel_size: 1.0,
            screen_width: WINDOW_WIDTH,
            screen_height: WINDOW_HEIGHT,
            scanline_intensity: 0.0,
            chromatic_aberration: 0.0,
            vignette_intensity: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

impl ExtractComponent for PixelationMaterial {
    type QueryData = &'static Self;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: bevy::ecs::query::QueryItem<'_, '_, Self::QueryData>) -> Option<Self::Out> {
        Some(*item)
    }
}

impl FullscreenMaterial for PixelationMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://klotzkoepfe/shaders/pixelation.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node2d::EndMainPassPostProcessing.intern(),
        ]
    }

    fn sub_graph() -> Option<InternedRenderSubGraph> {
        Some(Core2d.intern())
    }
}

/// Setup: erstellt eine Camera2d mit PixelationMaterial
pub fn setup_pixelation(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PixelationMaterial::default(),
        MainCamera,
    ));
}

/// Synchronisiert PixelationMaterial mit GameSettings
pub fn update_pixelation(
    settings: Res<GameSettings>,
    mut query: Query<&mut PixelationMaterial>,
) {
    for mut mat in query.iter_mut() {
        if settings.pixelation_enabled {
            mat.pixel_size = settings.pixel_size.max(1.0);
        } else {
            mat.pixel_size = 1.0;
        }
        if settings.retro_crt_enabled {
            mat.scanline_intensity = settings.scanline_intensity;
            mat.chromatic_aberration = settings.chromatic_aberration;
            mat.vignette_intensity = settings.vignette_intensity;
        } else {
            mat.scanline_intensity = 0.0;
            mat.chromatic_aberration = 0.0;
            mat.vignette_intensity = 0.0;
        }
        mat.screen_width = WINDOW_WIDTH;
        mat.screen_height = WINDOW_HEIGHT;
    }
}
