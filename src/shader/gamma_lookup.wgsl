struct ImageDisplay {
    window_size: vec2<f32>,
    pos: vec2<f32>,
    scale: f32,
    gamma: f32,
    scaling_mode: u32,
};

@group(0) @binding(0)
var<uniform> image_display : ImageDisplay;

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

// Map to a value dependant on the gamma value
fn gamma_lookup(colour: f32) -> f32 {
    let normalized = colour / 256.0;
    var inverse_gamma = 1.0 / image_display.gamma;
    return pow(normalized, inverse_gamma);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(gamma_lookup(in.clip_position.x), 0.0, 0.0, 1.0);
}