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
    cross_correlation: u32,
    clear_colour: vec4<f32>
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

fn gamma_correction(colour: vec4<f32>) -> vec4<f32> {
    var scaled = colour;
    var inverse_gamma = 1.0 / image_display.gamma;
    return vec4<f32>(
        pow(scaled.x, inverse_gamma),
        pow(scaled.y, inverse_gamma),
        pow(scaled.z, inverse_gamma),
        pow(scaled.w, inverse_gamma),
    );
}

fn sample(pos : vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, pos);
}

fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return gamma_correction(sample(in.clip_position.xy / (tex_size() * image_display.scale)));
}