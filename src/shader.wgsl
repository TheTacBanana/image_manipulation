@group(0) @binding(0)
var<uniform> dims: vec2<f32>;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct ImageDisplay {
    pos: vec2<f32>,
    scale: f32,
    gamma: f32,
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

fn screen_pos_to_tex_coord(pos : vec2<f32>) -> vec2<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_diffuse));
    return floor((pos.xy - image_display.pos) / image_display.scale) / tex_size;
}

fn gamma_correction(colour : vec4<f32>) -> vec4<f32> {
    var scaled = colour; // / 255.0;
    var inverse_gamma = 1.0 / image_display.gamma;
    return vec4<f32>(
        pow(scaled.x, inverse_gamma),
        pow(scaled.y, inverse_gamma),
        pow(scaled.z, inverse_gamma),
        pow(scaled.w, inverse_gamma),
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_pos = screen_pos_to_tex_coord(in.clip_position.xy);
    let sample = textureSample(t_diffuse, s_diffuse, tex_pos);

    if tex_pos.x < 0.0 || tex_pos.y < 0.0 || tex_pos.x > 1.0 || tex_pos.y > 1.0 {
        return vec4<f32>(0.0);
    } else {
        return gamma_correction(sample);
    };
}