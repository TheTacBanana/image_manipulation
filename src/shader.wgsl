@group(0) @binding(0)
var<uniform> dims: vec2<f32>;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct ImageDisplay {
    pos: vec2<f32>,
    scale: f32,
};

@group(2) @binding(0)
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos = in.clip_position;

    let tex_size = vec2<f32>(textureDimensions(t_diffuse));
    var scaled_pos = ((pos.xy - image_display.pos) * image_display.scale) / tex_size;

    let sample = textureSample(t_diffuse, s_diffuse, scaled_pos);

    if scaled_pos.x < 0.0 || scaled_pos.y < 0.0 || scaled_pos.x > 1.0 || scaled_pos.y > 1.0 {
        return vec4<f32>(0.0);
    } else {
        return sample;
    };
}