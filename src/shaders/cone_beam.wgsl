#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ConeBeamParams {
    color_inner: vec4<f32>,
    color_outer: vec4<f32>,
    time: f32,
    intensity: f32,    // 0..1, smooth ramped
    cone_angle: f32,
    beam_type: f32,    // 0 = flame, 1 = freeze
    // Treffer-Distanzen (normalisiert 0..1), 8 Werte in 2x vec4
    hit_distances_0: vec4<f32>,
    hit_distances_1: vec4<f32>,
    hit_count: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
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

// ===================== Hit Helpers =====================

// Liest Treffer-Distanz aus den 2 vec4s
fn get_hit_dist(index: i32) -> f32 {
    if index < 4 {
        if index == 0 { return params.hit_distances_0.x; }
        if index == 1 { return params.hit_distances_0.y; }
        if index == 2 { return params.hit_distances_0.z; }
        return params.hit_distances_0.w;
    } else {
        if index == 4 { return params.hit_distances_1.x; }
        if index == 5 { return params.hit_distances_1.y; }
        if index == 6 { return params.hit_distances_1.z; }
        return params.hit_distances_1.w;
    }
}

// Berechnet Splash-Intensitaet an einer Position basierend auf allen Treffern
fn hit_splash(dist: f32, t: f32, lateral: f32) -> f32 {
    var splash = 0.0;
    let count = i32(params.hit_count + 0.5);
    for (var i = 0; i < 8; i++) {
        if i >= count { break; }
        let hd = get_hit_dist(i);
        // Entfernung zum Trefferpunkt
        let d = abs(dist - hd);
        // Splash-Ring um den Trefferpunkt
        let ring = smoothstep(0.12, 0.0, d);
        // Seitliche Spritzer: staerker nahe am Treffer
        let side_spray = abs(lateral) * ring * 0.8;
        splash += ring * 0.7 + side_spray;
    }
    return min(splash, 1.5);
}

// Berechnet wie stark der Strahl hinter Treffern abgedaempft wird
fn hit_absorption(dist: f32) -> f32 {
    var absorption = 0.0;
    let count = i32(params.hit_count + 0.5);
    for (var i = 0; i < 8; i++) {
        if i >= count { break; }
        let hd = get_hit_dist(i);
        // Hinter dem Treffer: Strahl wird abgeschwaecht
        let behind = smoothstep(hd - 0.02, hd + 0.05, dist);
        absorption += behind * 0.15;
    }
    return min(absorption, 0.7);
}

// ===================== FLAME =====================

fn render_flame(raw_uv: vec2<f32>, t: f32, intensity: f32) -> vec4<f32> {
    let grid_size = 24.0;
    let uv = pixelate(raw_uv, grid_size);

    // UV mit Padding: Effekt nutzt nur inneren Bereich (~0.12..0.88)
    let padded_x = (uv.x - 0.12) / 0.76;
    let padded_y = (uv.y - 0.15) / 0.70;
    let dist = padded_x;
    let lateral = (padded_y - 0.5) * 2.0; // -1..1

    // Reichweite skaliert mit Intensity
    let reach = intensity;
    let reach_fade = 1.0 - smoothstep(reach * 0.85, reach, dist);

    // Treffer-Effekte
    let splash = hit_splash(dist, t, lateral);
    let absorption = hit_absorption(dist);

    // UV-Distortion: staerker zur Spitze hin + extra Turbulenz an Treffern
    let hit_turb = splash * 0.3;
    let distort_strength = dist * dist * 0.4 + hit_turb;
    let distort = vec2(
        fbm(uv * vec2(5.0, 3.0) + vec2(t * 3.0, t * 1.5)) - 0.5,
        fbm(uv * vec2(4.0, 6.0) + vec2(t * 2.0, t * 4.0)) - 0.5
    ) * distort_strength;

    let distorted_lateral = lateral + distort.y * 2.0;
    let distorted_dist = dist + distort.x * 0.3;

    // Flammenform: breitet sich aus + an Treffern breiter (Spritzer)
    let splash_spread = splash * 0.25;
    let spread = 0.08 + dist * 0.7 + splash_spread;
    let flame_edge = abs(distorted_lateral) / max(spread, 0.01);

    // Noise-basierte Rand-Maske
    let edge_noise = fbm(vec2(dist * 8.0 - t * 5.0, lateral * 4.0)) * 0.5;
    let flame_mask = 1.0 - smoothstep(0.5 + edge_noise, 1.0 + edge_noise * 0.5, flame_edge);

    // Distanz-Abfall + Absorption hinter Treffern
    let dist_fade = smoothstep(1.1, 0.6, distorted_dist) * (1.0 - absorption);
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

    // Treffer: heller Aufblitz (weiss-gelb)
    let splash_glow = splash * 0.5;
    color = vec4(color.rgb + vec3(splash_glow, splash_glow * 0.7, splash_glow * 0.2), color.a);

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

    // Reichweite skaliert mit Intensity
    let reach = intensity;
    let reach_fade = 1.0 - smoothstep(reach * 0.85, reach, dist);

    // Treffer-Effekte
    let splash = hit_splash(dist, t, lateral);
    let absorption = hit_absorption(dist);

    // Breiter, ungleichmaessiger Kegel - wolkenartig
    let slow_t = t * 0.8;
    let hit_turb = splash * 0.2;
    let cloud_distort = vec2(
        fbm(uv * vec2(3.0, 2.5) + vec2(slow_t * 0.7, slow_t * 0.3)) - 0.5,
        fbm(uv * vec2(2.5, 4.0) + vec2(slow_t * 0.5, slow_t * 0.9)) - 0.5
    );

    let distorted_lateral = lateral + cloud_distort.y * dist * 0.8;

    // Breite Wolkenform + Spritzer an Treffern
    let splash_spread = splash * 0.3;
    let spread = 0.05 + dist * 0.85 + cloud_distort.x * 0.2 + splash_spread;
    let cone_factor = abs(distorted_lateral) / max(spread, 0.01);

    // Wolkige Rand-Maske
    let cloud_noise = fbm(vec2(dist * 5.0 - slow_t * 2.0, lateral * 3.0 + slow_t * 0.5));
    let cloud_mask = 1.0 - smoothstep(0.4 + cloud_noise * 0.4, 1.0, cone_factor);

    // Distanz-Abfall + Absorption
    let dist_fade = smoothstep(1.1, 0.5, dist) * (1.0 - absorption);
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

    // Treffer: heller Aufblitz (weiss-blau)
    let splash_glow = splash * 0.4;
    color = vec4(color.rgb + vec3(splash_glow * 0.5, splash_glow * 0.7, splash_glow), color.a);

    // Kristall-Textur
    color = vec4(color.rgb * crystal_pattern, color.a);

    // Glitzer + extra Glitzer an Treffern
    let hit_glitter = glitter + splash * 0.3;
    color = vec4(color.rgb + vec3(hit_glitter, hit_glitter, hit_glitter * 1.2), color.a);

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
