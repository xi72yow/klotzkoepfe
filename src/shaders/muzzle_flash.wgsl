#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct MuzzleFlashParams {
    color_inner: vec4<f32>,
    color_outer: vec4<f32>,
    progress: f32,
    intensity: f32,
    _padding1: f32,
    _padding2: f32,
};

@group(2) @binding(0) var<uniform> params: MuzzleFlashParams;

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

fn pixelate(uv: vec2<f32>, grid: f32) -> vec2<f32> {
    return floor(uv * grid) / grid;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = params.progress;
    let intensity = params.intensity;

    let grid_size = 12.0;
    let raw_uv = (mesh.uv - vec2(0.5)) * 2.0;
    let uv = pixelate(raw_uv, grid_size);

    let cx = uv.x;
    let cy = uv.y;
    let dist_from_axis = abs(cy);

    // --- Feuer-Kern ---
    let base_width = 0.2 + cx * 0.4;
    let edge_wob = noise(vec2(cy * 5.0 + t * 10.0, cx * 3.0)) * 0.15;
    let cone_width = base_width + edge_wob;
    let in_cone = dist_from_axis < cone_width && cx > -0.15 && cx < 0.7;

    // --- Rauch: komplett frei, wilde dunkelgraue Flecken ---
    // Verschiedene Noise-Schichten die sich unabhaengig bewegen
    let s1 = noise(vec2(cx * 5.0 - t * 6.0, cy * 7.0 + t * 2.0) + vec2(1.3, 5.7));
    let s2 = noise(vec2(cx * 8.0 - t * 3.0, cy * 4.0 - t * 4.0) + vec2(9.2, 3.1));
    let s3 = noise(vec2(cx * 3.0 + t * 1.5, cy * 9.0 + t * 3.0) + vec2(4.8, 7.4));

    // Harte Thresholds fuer fleckigen Look - jede Schicht unabhaengig
    let patch1 = select(0.0, 1.0, s1 > 0.58);
    let patch2 = select(0.0, 1.0, s2 > 0.62);
    let patch3 = select(0.0, 1.0, s3 > 0.55);

    // Kombinieren: Flecken muessen sich ueberlappen fuer extra Unregelmaessigkeit
    var smoke = patch1 * (patch2 + patch3 * 0.5);
    smoke = min(smoke, 1.0);

    // Rauch entsteht am Lauf und treibt nach vorne weg
    // Leichte Tendenz nach vorne, aber auch seitlich und hinten moeglich
    let dist_center = length(uv);
    let drift_mask = 1.0 - smoothstep(0.6, 1.0, dist_center);
    smoke = smoke * drift_mask;

    // Rauch wird mit der Zeit staerker, Feuer geht
    let smoke_appear = smoothstep(0.05, 0.4, t);

    // Nichts?
    if !in_cone && smoke < 0.01 {
        discard;
    }

    // --- Feuer-Farbe ---
    let along = clamp(cx + 0.2, 0.0, 1.0);
    let across = dist_from_axis / max(cone_width, 0.01);
    let color_t = floor((along * 0.6 + across * 0.4) * 3.0) / 3.0;
    var fire_color = mix(params.color_inner, params.color_outer, color_t);

    let core = select(0.0, 1.0, along < 0.35 && across < 0.5);
    let core_fade = select(0.0, 1.0, t < 0.3);
    fire_color = mix(fire_color, vec4(1.0, 1.0, 0.9, 1.0), core * core_fade * 0.9);

    // --- Rauch-Farbe: dunkelgrau, ungleichmaessig ---
    let shade = 0.15 + hash(uv * 11.0) * 0.15; // 0.15..0.30 = richtig dunkel
    let smoke_color = vec4(shade, shade * 0.9, shade * 0.85, 1.0);

    // --- Feuer-Alpha ---
    var fire_alpha: f32;
    if t < 0.15 {
        fire_alpha = 1.0;
    } else {
        let fade_t = (t - 0.15) / 0.85;
        fire_alpha = 1.0 - fade_t * fade_t;
    }
    fire_alpha = fire_alpha * (1.0 - along * 0.3);

    // --- Rauch-Alpha ---
    let smoke_alpha = smoke * smoke_appear * (1.0 - t * t * 0.5) * 0.55;

    // --- Kombinieren ---
    var final_color: vec4<f32>;

    if in_cone && fire_alpha > 0.1 {
        // Feuer mit Rauch drueber gemischt
        let smoke_blend = smoke * smoke_appear * 0.4;
        final_color = mix(fire_color, smoke_color, smoke_blend);
        final_color.a = max(fire_alpha, smoke_alpha) * intensity;
    } else if smoke > 0.01 {
        // Nur Rauch
        final_color = smoke_color;
        final_color.a = smoke_alpha * intensity;
    } else {
        discard;
    }

    if final_color.a < 0.01 {
        discard;
    }

    return final_color;
}
