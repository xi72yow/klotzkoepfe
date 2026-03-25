#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ElementalParams {
    burn_intensity: f32,
    freeze_intensity: f32,
    time: f32,
    freeze_flash: f32,
    seed: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(2) @binding(0) var<uniform> params: ElementalParams;

fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn pixelate(uv: vec2<f32>, grid: f32) -> vec2<f32> {
    return floor(uv * grid) / grid;
}

// Feuer: wenige Flaemmchen mit individueller Farb- und Alpha-Variation
fn fire_effect(uv: vec2<f32>, t: f32, intensity: f32, seed: f32) -> vec4<f32> {
    let puv = pixelate(uv, 12.0);

    // Seed verschiebt Zeit und Zellenposition -> jeder Zombie flackert anders
    let offset_t = t + seed;
    let time_slot = floor(offset_t * 3.0);
    let cell_id = puv * 23.0 + vec2(seed * 7.13, seed * 3.71);
    let cell_lottery = hash(cell_id + vec2(time_slot * 1.7, time_slot * 0.3));

    // Weniger Partikel: ~5 max bei intensity=1 (12x12=144, ~90 body, 5/90=0.056)
    let threshold = 1.0 - intensity * 0.056;
    let is_flame = select(0.0, 1.0, cell_lottery > threshold);

    let dist = length(uv - vec2(0.5));
    let body_mask = select(0.0, 1.0, dist < 0.38);
    let flame = is_flame * body_mask;

    // Mehr Farbvariation: von gelb-orange bis tief-rot
    let color_var = hash(cell_id + vec2(time_slot * 0.5));
    let alpha_var = hash(cell_id + vec2(time_slot * 1.3, 7.0));
    let r = 1.0;
    let g = 0.15 + color_var * 0.6; // 0.15-0.75 statt 0.3-0.8
    let b_val = color_var * 0.05;

    // Alpha variiert pro Partikel: 0.5-1.0
    let alpha = flame * (0.5 + alpha_var * 0.5);
    return vec4(r, g, b_val, alpha);
}

// Eis: wenige Glitzer-Pixel mit Transparenz-Variation
fn ice_effect(uv: vec2<f32>, t: f32, intensity: f32, seed: f32) -> vec4<f32> {
    let puv = pixelate(uv, 12.0);

    let offset_t = t + seed;
    let time_slot = floor(offset_t * 3.5);
    let cell_id = puv * 37.0 + vec2(seed * 11.37, seed * 5.19);
    let cell_lottery = hash(cell_id + vec2(time_slot * 2.3, time_slot * 0.7));

    // Weniger Partikel: ~12 max bei intensity=1 (statt ~20)
    let threshold = 1.0 - intensity * 0.13;
    let is_sparkle = select(0.0, 1.0, cell_lottery > threshold);

    let dist = length(uv - vec2(0.5));
    let body_mask = select(0.0, 1.0, dist < 0.38);
    let sparkle = is_sparkle * body_mask;

    // Mehr Farbvariation: von weiss-blau bis tiefblau
    let color_var = hash(cell_id + vec2(time_slot * 0.7));
    let alpha_var = hash(cell_id + vec2(time_slot * 1.9, 13.0));
    let r = 0.5 + color_var * 0.5;  // 0.5-1.0
    let g = 0.7 + color_var * 0.3;  // 0.7-1.0
    let b_val = 0.85 + color_var * 0.15;

    // Alpha variiert: 0.4-0.9
    let alpha = sparkle * (0.4 + alpha_var * 0.5);
    return vec4(r, g, b_val, alpha);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let t = params.time;
    let burn = params.burn_intensity;
    let freeze = params.freeze_intensity;
    let flash = params.freeze_flash;
    let seed = params.seed;

    if burn < 0.01 && freeze < 0.01 && flash < 0.01 {
        discard;
    }

    let dist = length(uv - vec2(0.5));
    if dist > 0.5 {
        discard;
    }

    var color = vec4(0.0, 0.0, 0.0, 0.0);

    if burn > 0.01 {
        let fire = fire_effect(uv, t, burn, seed);
        color = vec4(
            color.r + fire.r * fire.a,
            color.g + fire.g * fire.a,
            color.b + fire.b * fire.a,
            max(color.a, fire.a)
        );
    }

    if freeze > 0.01 {
        let ice = ice_effect(uv, t, freeze, seed);
        color = vec4(
            mix(color.r, ice.r, ice.a),
            mix(color.g, ice.g, ice.a),
            mix(color.b, ice.b, ice.a),
            max(color.a, ice.a)
        );
    }

    // Freeze-Flash: heller weiss-blauer Blitz ueber den ganzen Body
    if flash > 0.01 {
        let body_mask = 1.0 - smoothstep(0.35, 0.48, dist);
        let flash_alpha = flash * flash * body_mask * 0.9;
        let flash_color = vec4(0.8, 0.9, 1.0, flash_alpha);
        color = vec4(
            mix(color.r, flash_color.r, flash_alpha),
            mix(color.g, flash_color.g, flash_alpha),
            mix(color.b, flash_color.b, flash_alpha),
            max(color.a, flash_alpha)
        );
    }

    if color.a < 0.01 {
        discard;
    }

    return color;
}
