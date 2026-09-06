#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// x: inner half-extent, y: wall thickness, z: corner radius (fractions of quad)
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> shape: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> fill_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> wall_color: vec4<f32>;
// x: 0 = fill, 1 = walls, y: obstacle count
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> style: vec4<f32>;
// (x, y, radius, 0) in uv space
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> obstacles: array<vec4<f32>, 16>;

fn rounded_box(p: vec2<f32>, half: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half + vec2<f32>(radius);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv - vec2<f32>(0.5);
    let aa = fwidth(p.x);
    let inner = rounded_box(p, vec2<f32>(shape.x), shape.z);
    let outer = inner - shape.y;

    var alpha: f32;
    var color: vec3<f32>;
    if style.x < 0.5 {
        alpha = 1.0 - smoothstep(-aa, aa, inner);
        color = fill_color.rgb;
    } else {
        alpha = (1.0 - smoothstep(-aa, aa, outer)) * smoothstep(-aa, aa, inner);
        for (var i = 0; i < i32(style.y); i++) {
            let circle = obstacles[i];
            let disc = length(in.uv - circle.xy) - circle.z;
            alpha = max(alpha, 1.0 - smoothstep(-aa, aa, disc));
        }
        color = wall_color.rgb;
    }

    return vec4<f32>(color, alpha);
}
