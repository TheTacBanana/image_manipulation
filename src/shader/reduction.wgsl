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

fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

fn sample(point : vec2<f32>) -> vec2<f32> {
    return textureSample(t_diffuse, s_diffuse, point + vec2<f32>(0.5)).xy;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let top_left_p = in.clip_position.xy * 2.0 / tex_size();

    let top_left = sample(top_left_p);
    let top_right = sample(top_left_p + vec2<f32>(1.0, 0.0));
    let bottom_left = sample(top_left_p + vec2<f32>(0.0, 1.0));
    let bottom_right = sample(top_left_p + vec2<f32>(1.0, 1.0));

    let min = min(min(top_left.x, top_right.x), min(bottom_left.x, bottom_right.x));
    let max = max(max(top_left.y, top_right.y), max(bottom_left.y, bottom_right.y));

    return vec4<f32>(min, max, 0.0, 1.0);
}