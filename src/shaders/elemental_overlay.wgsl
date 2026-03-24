#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ElementalParams {
    burn_intensity: f32,
    freeze_intensity: f32,
    time: f32,
    freeze_flash: f32,
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

// Feuer: wenige einzelne Flaemmchen-Pixel die flackern und verschwinden
fn fire_effect(uv: vec2<f32>, t: f32, intensity: f32) -> vec4<f32> {
    let puv = pixelate(uv, 14.0);

    let time_slot = floor(t * 3.5);
    let cell_lottery = hash(puv * 23.0 + vec2(time_slot * 1.7, time_slot * 0.3));

    // ~8 max bei intensity=1, ~1 bei intensity=0.1
    // 14x14=196 Zellen, ~120 im Body, 8/120=0.067
    let threshold = 1.0 - intensity * 0.067;
    let is_flame = select(0.0, 1.0, cell_lottery > threshold);

    let dist = length(uv - vec2(0.5));
    let body_mask = select(0.0, 1.0, dist < 0.38);
    let flame = is_flame * body_mask;

    let color_var = hash(puv * 11.0 + vec2(time_slot));
    let r = 1.0;
    let g = 0.3 + color_var * 0.5;
    let b_val = 0.02 + color_var * 0.08;

    let alpha = flame * 0.9;
    return vec4(r, g, b_val, alpha);
}

// Eis: wenige Glitzer-Pixel die kurz aufblitzen
fn ice_effect(uv: vec2<f32>, t: f32, intensity: f32) -> vec4<f32> {
    let puv = pixelate(uv, 14.0);

    let time_slot = floor(t * 4.0);
    let cell_lottery = hash(puv * 37.0 + vec2(time_slot * 2.3, time_slot * 0.7));

    // ~20 max bei intensity=1
    let threshold = 1.0 - intensity * 0.17;
    let is_sparkle = select(0.0, 1.0, cell_lottery > threshold);

    let dist = length(uv - vec2(0.5));
    let body_mask = select(0.0, 1.0, dist < 0.38);
    let sparkle = is_sparkle * body_mask;

    let color_var = hash(puv * 19.0 + vec2(time_slot));
    let r = 0.65 + color_var * 0.35;
    let g = 0.8 + color_var * 0.2;
    let b_val = 1.0;

    let alpha = sparkle * 0.85;
    return vec4(r, g, b_val, alpha);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let t = params.time;
    let burn = params.burn_intensity;
    let freeze = params.freeze_intensity;
    let flash = params.freeze_flash;

    if burn < 0.01 && freeze < 0.01 && flash < 0.01 {
        discard;
    }

    let dist = length(uv - vec2(0.5));
    if dist > 0.5 {
        discard;
    }

    var color = vec4(0.0, 0.0, 0.0, 0.0);

    if burn > 0.01 {
        let fire = fire_effect(uv, t, burn);
        color = vec4(
            color.r + fire.r * fire.a,
            color.g + fire.g * fire.a,
            color.b + fire.b * fire.a,
            max(color.a, fire.a)
        );
    }

    if freeze > 0.01 {
        let ice = ice_effect(uv, t, freeze);
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
        let flash_alpha = flash * flash * body_mask * 0.9; // quadratisch fuer schnelles Abklingen
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
