#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct PixelationParams {
    pixel_size: f32,
    screen_width: f32,
    screen_height: f32,
    _padding: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> params: PixelationParams;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if (params.pixel_size <= 1.01) {
        return textureSample(screen_texture, texture_sampler, in.uv);
    }
    let resolution = vec2<f32>(params.screen_width, params.screen_height);
    let grid = resolution / params.pixel_size;
    let pixelated_uv = (floor(in.uv * grid) + 0.5) / grid;
    return textureSample(screen_texture, texture_sampler, pixelated_uv);
}
