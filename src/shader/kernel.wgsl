@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct ImageDisplay {
    window_size: vec2<f32>,
    pos: vec2<f32>,
    scale: f32,
    gamma: f32,
    scaling_mode: u32,
    global_min_max: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> image_display : ImageDisplay;

@group(2) @binding(0)
var<storage> laplacian : array<i32>;

// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

fn apply_kernel(pos: vec2<f32>) -> vec4<f32> {
    var s = vec3<f32>(0.0);
    for (var row = -2; row < 3; row += 1) {
        for (var col = -2; col < 3; col += 1) {
            let i = (row + 2) * 5 + (col + 2);
            let sample_pos = pos + vec2<f32>(f32(row), f32(col));
            s += sample(sample_pos).xyz * f32(laplacian[i]);
        }
    }
    return vec4<f32>(norm(s.x), norm(s.y), norm(s.z), 1.0);
}

fn norm(in: f32) -> f32 {
    return max(0.0, min(1.0, (in / 256.0) + 0.5));
}

fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

fn sample(pos : vec2<f32>) -> vec4<f32> {
    let clamped = min(max(vec2<i32>(pos), vec2<i32>(0)), vec2<i32>(tex_size()));
    let transformed = (vec2<f32>(clamped) + vec2<f32>(0.5)) / tex_size();
    return textureSample(t_diffuse, s_diffuse, transformed);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return (sample(in.clip_position.xy) / 512.0) + 0.5;
    // return (apply_kernel(in.clip_position.xy) / 512.0) + 0.5;
    return apply_kernel(in.clip_position.xy);
}