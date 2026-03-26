#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct PostProcessParams {
    pixel_size: f32,
    screen_width: f32,
    screen_height: f32,
    scanline_intensity: f32,
    chromatic_aberration: f32,
    vignette_intensity: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> params: PostProcessParams;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    let resolution = vec2<f32>(params.screen_width, params.screen_height);

    // === Pixelation ===
    if (params.pixel_size > 1.01) {
        let grid = resolution / params.pixel_size;
        uv = (floor(uv * grid) + 0.5) / grid;
    }

    // Kein CRT aktiv -> frueh raus
    let has_crt = params.scanline_intensity > 0.001
               || params.chromatic_aberration > 0.001
               || params.vignette_intensity > 0.001;
    if (!has_crt) {
        return textureSample(screen_texture, texture_sampler, uv);
    }

    // === Chromatic Aberration ===
    let ca_offset = params.chromatic_aberration / resolution.x;
    let r = textureSample(screen_texture, texture_sampler, vec2<f32>(uv.x + ca_offset, uv.y)).r;
    let g = textureSample(screen_texture, texture_sampler, uv).g;
    let b = textureSample(screen_texture, texture_sampler, vec2<f32>(uv.x - ca_offset, uv.y)).b;
    var color = vec3<f32>(r, g, b);

    // === Scanlines ===
    let screen_y = uv.y * resolution.y;
    let scanline = 1.0 - params.scanline_intensity * (0.5 + 0.5 * sin(screen_y * 3.14159 * 2.0));
    color *= scanline;

    // === Vignette ===
    let center = uv - 0.5;
    let dist = dot(center, center);
    let vignette = 1.0 - dist * params.vignette_intensity * 4.0;
    color *= clamp(vignette, 0.0, 1.0);

    return vec4<f32>(color, 1.0);
}
