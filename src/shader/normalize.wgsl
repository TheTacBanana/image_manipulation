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
};

@group(1) @binding(0)
var<uniform> image_display : ImageDisplay;

@group(2) @binding(0)
var mini_max_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var mini_max_sampler: sampler;

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


// Get the size of the texture into the shader
fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

// Sample the texture at the pixel coordinate
fn sample_pixel(pixel : vec2<i32>) -> vec4<f32> {
    let clamped = min(max(vec2<i32>(pixel), vec2<i32>(0)), vec2<i32>(tex_size()));
    let transformed = (vec2<f32>(clamped) + vec2<f32>(0.5)) / tex_size();
    return textureSample(t_diffuse, s_diffuse, transformed);
}

// Get the min and max from the bound texture, iterating over
// each mapped chunk to get the global min-max
fn min_and_max() -> vec2<f32> {
    var mini = 1.0;
    var maxi = 0.0;

    let size = textureDimensions(mini_max_diffuse);
    for (var row = 0; row < i32(size.y); row += 1) {
        for (var col = 0; col < i32(size.x); col += 1) {
            let point = vec2<f32>(f32(col), f32(row)) + vec2<f32>(0.5);
            let s = textureSample(mini_max_diffuse, mini_max_sampler, point / vec2<f32>(size)).xyz;
            mini = min(mini, s.x);
            maxi = max(maxi, s.y);
        }
    }
    return vec2<f32>(unnorm(mini), unnorm(maxi));
}

// Apply normalization on a colour
fn normalize(colour: vec4<f32>) -> vec4<f32> {
    let min_maxi = min_and_max();
    return vec4<f32>((colour.xyz - min_maxi.x) / (min_maxi.y - min_maxi.x), 1.0);
}

// Un-Normalize 0.0-1.0 to -128.0-128.0
fn unnorm(in: f32) -> f32 {
    return (in - 0.5) * 256.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let s = sample_pixel(vec2<i32>(in.clip_position.xy));
    let normed = vec4<f32>(unnorm(s.x), unnorm(s.y), unnorm(s.z), 1.0);
    return normalize(normed);
}