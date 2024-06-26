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
var gamma_lut_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var gamma_lut_sampler: sampler;

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

// Lookup the value in the lookup table
fn sample_lookup(i : f32) -> f32 {
    let transformed = round(i * 256.0);
    return textureSample(gamma_lut_diffuse, gamma_lut_sampler, vec2<f32>((transformed + 0.5) / 256.0, 0.5)).x;
}

// Apply gamma correction to a colour
fn gamma_correction(colour: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        sample_lookup(colour.x),
        sample_lookup(colour.y),
        sample_lookup(colour.z),
        1.0
    );
}

// Get the sample of the texture in at a pixel
fn sample(pos : vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, (pos + vec2<f32>(0.5)) / vec2<f32>(textureDimensions(t_diffuse)));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return gamma_correction(sample(in.clip_position.xy));
}