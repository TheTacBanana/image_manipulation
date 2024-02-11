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
    scaling_mode: u32,
    cross_correlation: u32,
    _pad1: f32,
    _pad2: f32,
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

fn nearest_neighbour(pos: vec2<f32>) -> vec4<f32> {
    var tex_pos = screen_pos_to_tex_coord(pos);
    let sample = textureSample(t_diffuse, s_diffuse, tex_pos);
    if !(tex_pos.x < 0.0 || tex_pos.y < 0.0 || tex_pos.x > 1.0 || tex_pos.y > 1.0) {
        return sample;
    } else {
        return vec4<f32>(0.0);
    }
}

fn billinear_filtering(pos: vec2<f32>) -> vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_diffuse));

    let right = vec2<f32>(image_display.scale, 0.0);
    let down = vec2<f32>(0.0, image_display.scale);

    let up = textureSample(t_diffuse, s_diffuse, screen_pos_to_tex_coord(pos - down));
    let ri = textureSample(t_diffuse, s_diffuse, screen_pos_to_tex_coord(pos + right));
    let le = textureSample(t_diffuse, s_diffuse, screen_pos_to_tex_coord(pos + down));
    let dow = textureSample(t_diffuse, s_diffuse, screen_pos_to_tex_coord(pos - right));

    return (up + ri + le + dow) / 4.0;
}

fn screen_pos_to_tex_coord(pos: vec2<f32>) -> vec2<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_diffuse));
    return floor((pos.xy - image_display.pos + (tex_size * image_display.scale / 2.0)) / image_display.scale) / tex_size;
}

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

fn to_pixel()

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return ((vec4<f32>(screen_pos_to_tex_coord(in.clip_position.xy), 0.0, 0.0) % 1.0) + vec4<f32>(1.0)) % 1.0;
    // switch image_display.scaling_mode {
    //     case 0u: {
    //         return gamma_correction(nearest_neighbour(in.clip_position.xy));
    //     }
    //     case 1u: {
    //         return gamma_correction(billinear_filtering(in.clip_position.xy));
    //     }
    //     default: {
    //         return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    //     }
    // }
}