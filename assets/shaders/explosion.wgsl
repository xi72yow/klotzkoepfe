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

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (mesh.uv - vec2(0.5)) * 2.0; // -1..1
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);
    let t = params.progress;
    let level_f = params.level;

    // Alles ausserhalb des Einheitskreises sofort verwerfen
    if dist > 1.0 {
        discard;
    }

    // --- Expansion: ease-out quadratisch ---
    var expand: f32;
    if t < 0.25 {
        let phase = t / 0.25;
        expand = 1.0 - (1.0 - phase) * (1.0 - phase); // ease-out
    } else if t < 0.5 {
        expand = 1.0;
    } else {
        let phase = (t - 0.5) / 0.5;
        expand = 1.0 - phase * 0.4; // langsam schrumpfen
    }

    // --- Dynamischer Feuerball-Rand mit animiertem Noise ---
    // Polar-Noise: verzerrt den Rand winkelabhaengig fuer organische Form
    let polar_noise = fbm(vec2(angle * 2.0 + t * 4.0, t * 6.0)) * 0.25;
    // Radiales Noise: kleine Beulen/Dellen
    let radial_noise = fbm(uv * (3.0 + level_f) + vec2(t * 5.0, -t * 3.0)) * 0.2;
    // Feines Detail-Noise fuer Flammen-Textur
    let detail = noise(uv * (8.0 + level_f * 3.0) - vec2(t * 8.0, t * 6.0)) * 0.1;

    let fire_radius = expand * 0.7;
    let fire_edge = fire_radius + polar_noise + radial_noise + detail;

    // Weicher Rand
    let fire_mask = smoothstep(fire_edge, fire_edge * 0.2, dist);

    // --- Farbe ---
    // Innen -> Aussen: weiss-gelb -> orange -> rot-dunkel
    let color_t = smoothstep(0.0, fire_edge * 0.9, dist);
    var fire_color = mix(params.color_inner, params.color_outer, color_t * color_t);

    // Heller Kern pulsiert leicht
    let core_pulse = 0.5 + 0.5 * sin(t * 20.0);
    let core_bright = smoothstep(0.35, 0.0, dist) * smoothstep(0.35, 0.0, t);
    fire_color = mix(fire_color, vec4(1.0, 1.0, 0.9, 1.0), core_bright * (0.5 + 0.2 * core_pulse));

    // Flammen-Textur: dunklere Streifen im Feuer
    let flame_tex = fbm(uv * 5.0 + vec2(t * 7.0, t * -4.0));
    fire_color = fire_color * (0.7 + 0.3 * flame_tex);

    // --- Shockwave Ring ---
    let ring_speed = 1.2 + 0.3 * level_f;
    let ring_radius = t * ring_speed;
    let ring_width = (0.08 + 0.02 * level_f) * (1.0 - t);
    let ring_dist = abs(dist - ring_radius);
    let ring_mask = smoothstep(ring_width, 0.0, ring_dist) * (1.0 - t * t) * 0.5;
    let ring_color = vec4(1.0, 0.65, 0.15, ring_mask);

    // --- Kombination ---
    var final_color = fire_color * fire_mask;
    final_color = vec4(
        final_color.rgb + ring_color.rgb * ring_color.a,
        max(final_color.a, ring_color.a)
    );

    // Global fade
    let fade = smoothstep(1.0, 0.5, t);
    final_color.a = final_color.a * fade;

    if final_color.a < 0.01 {
        discard;
    }

    return final_color;
}
