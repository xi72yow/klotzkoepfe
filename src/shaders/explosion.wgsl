#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ExplosionParams {
    color_inner: vec4<f32>,
    color_outer: vec4<f32>,
    progress: f32,
    level: f32,
    _padding1: f32,
    _padding2: f32,
};

@group(2) @binding(0) var<uniform> params: ExplosionParams;

// Hash-Noise Funktionen
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i + vec2(0.0, 0.0)), hash(i + vec2(1.0, 0.0)), u.x),
        mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x),
        u.y
    );
}

// Fraktales Noise (mehrere Oktaven fuer organischen Look)
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;
    for (var i = 0; i < 4; i++) {
        value += amplitude * noise(pos);
        pos = pos * 2.1;
        amplitude *= 0.5;
    }
    return value;
}

// UV-Koordinaten auf Pixel-Grid snappen
fn pixelate(uv: vec2<f32>, grid: f32) -> vec2<f32> {
    return floor(uv * grid) / grid;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = params.progress;
    let level_f = params.level;

    // Pixel-Grid: grobes Raster fuer blockigen Look
    let grid_size = 24.0 + level_f * 4.0;
    let raw_uv = (mesh.uv - vec2(0.5)) * 2.0; // -1..1
    let uv = pixelate(raw_uv, grid_size);
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);

    // Alles ausserhalb des Einheitskreises sofort verwerfen
    if dist > 1.0 {
        discard;
    }

    // --- Expansion: ease-out quadratisch ---
    var expand: f32;
    if t < 0.25 {
        let phase = t / 0.25;
        expand = 1.0 - (1.0 - phase) * (1.0 - phase);
    } else if t < 0.5 {
        expand = 1.0;
    } else {
        let phase = (t - 0.5) / 0.5;
        expand = 1.0 - phase * 0.4;
    }

    // --- Grober Noise fuer blockige Feuerballen ---
    // Weniger Oktaven, niedrigere Frequenz = groessere Bloecke
    let polar_noise = noise(vec2(angle * 1.5 + t * 3.0, t * 4.0)) * 0.3;
    let radial_noise = noise(uv * (2.0 + level_f) + vec2(t * 3.0, -t * 2.0)) * 0.25;

    let fire_radius = expand * 0.7;
    let fire_edge = fire_radius + polar_noise + radial_noise;

    // Harter Rand statt smoothstep
    let fire_mask = select(0.0, 1.0, dist < fire_edge);

    // --- Farbe: gestufte Farbbaender statt smooth gradient ---
    let color_raw = dist / max(fire_edge, 0.01);
    // 4 diskrete Farbstufen
    let color_t = floor(color_raw * 4.0) / 4.0;
    var fire_color = mix(params.color_inner, params.color_outer, color_t);

    // Heller Kern
    let core_mask = select(0.0, 1.0, dist < fire_edge * 0.25);
    let core_fade = select(0.0, 1.0, t < 0.3);
    fire_color = mix(fire_color, vec4(1.0, 1.0, 0.9, 1.0), core_mask * core_fade * 0.7);

    // Blockige Flammen-Textur
    let flame_tex = noise(uv * 3.0 + vec2(t * 5.0, t * -3.0));
    let flame_step = floor(flame_tex * 3.0) / 3.0;
    fire_color = fire_color * (0.65 + 0.35 * flame_step);

    // --- Shockwave Ring (auch pixelig) ---
    let ring_speed = 1.2 + 0.3 * level_f;
    let ring_radius = t * ring_speed;
    let ring_width = (0.1 + 0.03 * level_f) * (1.0 - t);
    let ring_dist = abs(dist - ring_radius);
    let ring_mask = select(0.0, 1.0, ring_dist < ring_width) * (1.0 - t * t) * 0.5;
    let ring_color = vec4(1.0, 0.65, 0.15, ring_mask);

    // --- Kombination ---
    var final_color = fire_color * fire_mask;
    final_color = vec4(
        final_color.rgb + ring_color.rgb * ring_color.a,
        max(final_color.a, ring_color.a)
    );

    // Global fade - auch gestuft
    let fade = select(0.0, 1.0, t < 0.7) * (1.0 - step(0.9, t) * 0.5);
    final_color.a = final_color.a * fade;

    if final_color.a < 0.01 {
        discard;
    }

    return final_color;
}
