// The field texture is alpha-blended blobs on transparent black: `a` is
// coverage, `rgb` is premultiplied speed colour.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var field_texture: texture_2d<f32>;
@group(2) @binding(1) var field_sampler: sampler;
// x: threshold, y: edge softness, z: rim width, w: rim brightness
@group(2) @binding(2) var<uniform> params: vec4<f32>;
// x: tank half-extent (fraction of quad), y: normal strength,
// z: gradient radius (texels), w: lit depth past the threshold
@group(2) @binding(3) var<uniform> clip: vec4<f32>;
// x: diffuse strength, y: specular strength, z: shininess, w: texel size in uv
@group(2) @binding(4) var<uniform> lighting: vec4<f32>;
// x: corner radius (fraction of quad), y: glow
@group(2) @binding(5) var<uniform> style: vec4<f32>;

fn rounded_box(p: vec2<f32>, half: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Upper left; uv.y points down.
const LIGHT_DIR: vec3<f32> = vec3<f32>(-0.45, -0.65, 0.6);
// Depth darkening, as a fraction of tank height from the top.
const DEPTH_START: f32 = 0.45;
const DEPTH_DARKEN: f32 = 0.45;

fn coverage_at(uv: vec2<f32>) -> f32 {
    return textureSample(field_texture, field_sampler, uv).a;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample before any discard: sampling needs uniform control flow.
    let sample = textureSample(field_texture, field_sampler, in.uv);
    let field = sample.a;
    let d = vec2<f32>(clip.z * lighting.w, 0.0);
    let gradient = vec2<f32>(
        coverage_at(in.uv + d.xy) - coverage_at(in.uv - d.xy),
        coverage_at(in.uv + d.yx) - coverage_at(in.uv - d.yx),
    );

    let p = in.uv - vec2<f32>(0.5);
    let aa = fwidth(p.x);
    let tank = rounded_box(p, vec2<f32>(clip.x), style.x);
    if tank > aa {
        discard;
    }

    let threshold = params.x;
    let softness = params.y;
    var coverage = smoothstep(threshold - softness, threshold + softness, field);
    coverage = coverage * (1.0 - smoothstep(-aa, aa, tank));
    if coverage <= 0.001 {
        discard;
    }

    let base = sample.rgb / max(field, 0.001);
    let rim = 1.0 - smoothstep(threshold, threshold + params.z, field);
    var color = mix(base, vec3<f32>(1.0), rim * params.w);

    // Both lighting terms vanish where the field is flat.
    let normal = normalize(vec3<f32>(-gradient * clip.y, 1.0));
    let light = normalize(LIGHT_DIR);
    let half_vector = normalize(light + vec3<f32>(0.0, 0.0, 1.0));

    // Facing-times-tilt rather than Lambert, so steep edges aren't all dark.
    let tilt = 1.0 - normal.z;
    let outward = normal.xy / max(length(normal.xy), 0.001);
    let facing = dot(outward, normalize(light.xy));
    let diffuse = facing * tilt;

    let specular = max(
        pow(max(dot(normal, half_vector), 0.0), lighting.z) - pow(half_vector.z, lighting.z),
        0.0,
    );

    let lit = 1.0 - smoothstep(threshold, threshold + clip.w, field);
    color = color * (1.0 + lighting.x * diffuse * lit);
    color = color + vec3<f32>(lighting.y * specular * lit);

    let tank_fraction = (in.uv.y - (0.5 - clip.x)) / (2.0 * clip.x);
    let depth = smoothstep(DEPTH_START, 1.0, tank_fraction);
    color = color * (1.0 - DEPTH_DARKEN * depth);

    // Past 1.0 for bloom.
    let fast = smoothstep(0.5, 0.95, luminance(base));
    color = color * (1.0 + style.y * fast);

    return vec4<f32>(color, coverage);
}
