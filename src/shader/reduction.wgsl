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

fn min_in_vec(colour : vec2<f32>) -> f32 {
    return min(colour.x, colour.y);
}

fn max_in_vec(colour : vec2<f32>) -> f32 {
    return max(colour.x, colour.y);
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

    let min = min(min(min_in_vec(top_left), min_in_vec(top_right)), min(min_in_vec(bottom_left), min_in_vec(bottom_right)));
    let max = max(max(max_in_vec(top_left), max_in_vec(top_right)), max(max_in_vec(bottom_left), max_in_vec(bottom_right)));

    return vec4<f32>(min, max, 0.0, 1.0);
}