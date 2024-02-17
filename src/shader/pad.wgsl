@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// struct ImageDisplay {
//     window_size: vec2<f32>,
//     pos: vec2<f32>,
//     scale: f32,
//     gamma: f32,
//     scaling_mode: u32,
//     global_min_max: vec2<f32>,
// };

// @group(1) @binding(0)
// var<uniform> image_display : ImageDisplay;

// @group(2) @binding(0)
// var<storage> laplacian : array<i32>;

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

fn min_in_vec(colour : vec3<f32>) -> f32 {
    return min(colour.x, min(colour.y, colour.z));
}

fn max_in_vec(colour : vec3<f32>) -> f32 {
    return max(colour.x, max(colour.y, colour.z));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let point = in.clip_position.xy / tex_size();
    let sample = textureSample(t_diffuse, s_diffuse, point);

    let min = min_in_vec(sample.xyz);
    let max = min_in_vec(sample.xyz);

    return vec4<f32>(min, max, 0.0, 1.0);
}