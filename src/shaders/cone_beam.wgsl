#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ConeBeamParams {
    color_inner: vec4<f32>,
    color_outer: vec4<f32>,
    time: f32,
    intensity: f32,    // 0..1, smooth ramped
    cone_angle: f32,
    beam_type: f32,    // 0 = flame, 1 = freeze
};

@group(2) @binding(0) var<uniform> params: ConeBeamParams;

// ===================== Noise Helpers =====================

fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let px = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
    let pp = px + dot(px, px.yzx + 33.33);
    return fract((pp.xx + pp.yz) * pp.zy);
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

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;
    for (var i = 0; i < 4; i++) {
        value += amplitude * noise(pos);
        pos = pos * 2.1 + vec2(1.7, 3.2);
        amplitude *= 0.5;
    }
    return value;
}

// Voronoi fuer Eiskristall-Muster
fn voronoi(p: vec2<f32>) -> vec2<f32> {
    let n = floor(p);
    let f = fract(p);
    var min_dist = 1.0;
    var second_dist = 1.0;
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2(f32(i), f32(j));
            let o = hash2(n + g);
            let r = g + o - f;
            let d = dot(r, r);
            if d < min_dist {
                second_dist = min_dist;
                min_dist = d;
            } else if d < second_dist {
                second_dist = d;
            }
        }
    }
    return vec2(sqrt(min_dist), sqrt(second_dist));
}

// Pixelate
fn pixelate(uv: vec2<f32>, grid: f32) -> vec2<f32> {
    return floor(uv * grid) / grid;
}

// ===================== FLAME =====================

fn render_flame(raw_uv: vec2<f32>, t: f32, intensity: f32) -> vec4<f32> {
    let grid_size = 24.0;
    let uv = pixelate(raw_uv, grid_size);

    // UV mit Padding: Effekt nutzt nur inneren Bereich (~0.12..0.88)
    // Damit Noise-Verzerrungen nicht am Meshrand abgeschnitten werden
    let padded_x = (uv.x - 0.12) / 0.76; // 0..1 im Effektbereich
    let padded_y = (uv.y - 0.15) / 0.70;
    let dist = padded_x;
    let lateral = (padded_y - 0.5) * 2.0; // -1..1

    // Reichweite skaliert mit Intensity -> Anlauf/Ablauf-Animation
    let reach = intensity; // 0..1, Flamme "schiesst raus"
    // Alles jenseits der Reichweite wird ausgeblendet
    let reach_fade = 1.0 - smoothstep(reach * 0.85, reach, dist);

    // UV-Distortion: staerker zur Spitze hin
    let distort_strength = dist * dist * 0.4;
    let distort = vec2(
        fbm(uv * vec2(5.0, 3.0) + vec2(t * 3.0, t * 1.5)) - 0.5,
        fbm(uv * vec2(4.0, 6.0) + vec2(t * 2.0, t * 4.0)) - 0.5
    ) * distort_strength;

    let distorted_lateral = lateral + distort.y * 2.0;
    let distorted_dist = dist + distort.x * 0.3;

    // Flammenform: breitet sich aus, noise-basierter Rand
    let spread = 0.08 + dist * 0.7;
    let flame_edge = abs(distorted_lateral) / max(spread, 0.01);

    // Noise-basierte Rand-Maske (organisch ausfransend)
    let edge_noise = fbm(vec2(dist * 8.0 - t * 5.0, lateral * 4.0)) * 0.5;
    let flame_mask = 1.0 - smoothstep(0.5 + edge_noise, 1.0 + edge_noise * 0.5, flame_edge);

    // Distanz-Abfall
    let dist_fade = smoothstep(1.1, 0.6, distorted_dist);
    let start_fade = smoothstep(-0.05, 0.08, dist);

    // Turbulenz-Textur
    let turb1 = fbm(uv * vec2(6.0, 4.0) - vec2(t * 6.0, t * 2.0));
    let turb2 = noise(uv * vec2(10.0, 8.0) - vec2(t * 8.0, t * 3.0));

    // Feuer-Farbrampe
    let core_factor = 1.0 - smoothstep(0.0, 0.35, abs(distorted_lateral) / max(spread, 0.01));
    let dist_color = smoothstep(0.0, 0.7, dist);

    let white_hot = vec4(1.0, 1.0, 0.85, 1.0);
    let yellow = params.color_inner;
    let orange = vec4(1.0, 0.5, 0.05, 0.9);
    let red = params.color_outer;

    var color: vec4<f32>;
    let ramp = dist_color * (1.0 - core_factor * 0.5);
    if ramp < 0.25 {
        color = mix(white_hot, yellow, ramp * 4.0);
    } else if ramp < 0.5 {
        color = mix(yellow, orange, (ramp - 0.25) * 4.0);
    } else {
        color = mix(orange, red, (ramp - 0.5) * 2.0);
    }

    // Flackern
    let flicker = 0.75 + 0.25 * turb2;
    color = vec4(color.rgb * flicker, color.a);

    // Glutfetzen
    let ember = smoothstep(0.72, 0.78, turb1) * (1.0 - dist) * 0.6;
    color = vec4(color.rgb + vec3(ember, ember * 0.5, 0.0), color.a);

    let alpha = flame_mask * dist_fade * start_fade * reach_fade;

    if alpha < 0.01 {
        discard;
    }

    return vec4(color.rgb, alpha);
}

// ===================== FREEZE =====================

fn render_freeze(raw_uv: vec2<f32>, t: f32, intensity: f32) -> vec4<f32> {
    let grid_size = 20.0;
    let uv = pixelate(raw_uv, grid_size);

    // UV mit Padding
    let padded_x = (uv.x - 0.10) / 0.80;
    let padded_y = (uv.y - 0.12) / 0.76;
    let dist = padded_x;
    let lateral = (padded_y - 0.5) * 2.0;

    // Reichweite skaliert mit Intensity -> Anlauf/Ablauf
    let reach = intensity;
    let reach_fade = 1.0 - smoothstep(reach * 0.85, reach, dist);

    // Breiter, ungleichmaessiger Kegel - wolkenartig
    let slow_t = t * 0.8;
    let cloud_distort = vec2(
        fbm(uv * vec2(3.0, 2.5) + vec2(slow_t * 0.7, slow_t * 0.3)) - 0.5,
        fbm(uv * vec2(2.5, 4.0) + vec2(slow_t * 0.5, slow_t * 0.9)) - 0.5
    );

    let distorted_lateral = lateral + cloud_distort.y * dist * 0.8;

    // Breite Wolkenform
    let spread = 0.05 + dist * 0.85 + cloud_distort.x * 0.2;
    let cone_factor = abs(distorted_lateral) / max(spread, 0.01);

    // Wolkige Rand-Maske
    let cloud_noise = fbm(vec2(dist * 5.0 - slow_t * 2.0, lateral * 3.0 + slow_t * 0.5));
    let cloud_mask = 1.0 - smoothstep(0.4 + cloud_noise * 0.4, 1.0, cone_factor);

    // Distanz-Abfall
    let dist_fade = smoothstep(1.1, 0.5, dist);
    let start_fade = smoothstep(-0.05, 0.06, dist);

    // Voronoi-Eiskristall-Textur
    let crystal_uv = uv * vec2(8.0, 6.0) + vec2(-slow_t * 1.5, slow_t * 0.3);
    let vor = voronoi(crystal_uv);
    let cell_edge = smoothstep(0.0, 0.15, vor.y - vor.x);
    let crystal_pattern = mix(0.6, 1.0, cell_edge);

    // Glitzernde Punkte
    let glitter_uv = uv * vec2(16.0, 12.0);
    let glitter_cell = floor(glitter_uv);
    let glitter_phase = hash(glitter_cell) * 6.28 + t * 4.0;
    let glitter_brightness = pow(max(sin(glitter_phase), 0.0), 12.0);
    let glitter = glitter_brightness * cloud_mask * 0.7 * step(0.7, hash(glitter_cell + vec2(7.0, 13.0)));

    // Farbrampe
    let core_factor = 1.0 - smoothstep(0.0, 0.4, abs(distorted_lateral) / max(spread, 0.01));
    let ramp = dist * (1.0 - core_factor * 0.4);

    var color: vec4<f32>;
    let ice_white = vec4(0.92, 0.97, 1.0, 1.0);
    let ice_light = params.color_inner;
    let ice_mid = vec4(0.4, 0.7, 1.0, 0.85);
    let ice_deep = params.color_outer;

    if ramp < 0.3 {
        color = mix(ice_white, ice_light, ramp / 0.3);
    } else if ramp < 0.6 {
        color = mix(ice_light, ice_mid, (ramp - 0.3) / 0.3);
    } else {
        color = mix(ice_mid, ice_deep, (ramp - 0.6) / 0.4);
    }

    // Kristall-Textur
    color = vec4(color.rgb * crystal_pattern, color.a);

    // Glitzer
    color = vec4(color.rgb + vec3(glitter, glitter, glitter * 1.2), color.a);

    // Wolken-Flimmern
    let cloud_flicker = 0.85 + 0.15 * noise(uv * vec2(6.0, 4.0) - vec2(slow_t * 1.2, slow_t * 0.8));
    color = vec4(color.rgb * cloud_flicker, color.a);

    let alpha = cloud_mask * dist_fade * start_fade * reach_fade * 0.85;

    if alpha < 0.01 {
        discard;
    }

    return vec4(color.rgb, alpha);
}

// ===================== Fragment Entry =====================

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = params.time;
    let intensity = params.intensity;

    if intensity < 0.005 {
        discard;
    }

    if params.beam_type < 0.5 {
        return render_flame(mesh.uv, t, intensity);
    } else {
        return render_freeze(mesh.uv, t, intensity);
    }
}
