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

fn screen_pos_to_tex_coord(pos: vec2<f32>) -> vec2<f32> {
    return ((pos - image_display.pos - image_display.window_size / 2.0 + tex_size() / 2.0) / tex_size());
}

fn sample(pos : vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, pos);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_pos = screen_pos_to_tex_coord(in.clip_position.xy);
    if tex_pos.x < 0.0 || tex_pos.y < 0.0 || tex_pos.x > 1.0 || tex_pos.y > 1.0 {
        discard;
    }
    return sample(tex_pos);
}