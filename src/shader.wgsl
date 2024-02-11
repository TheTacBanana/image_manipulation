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
    clear_colour: vec4<f32>
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

fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

fn nearest_neighbour(pos: vec2<f32>) -> vec4<f32> {
    let tex_pos = screen_pos_to_tex_coord(pos);
    let real_step = 1.0 / tex_size();

    let top_left = floor(tex_pos / real_step);
    let rounded = round((tex_pos % real_step) / real_step - vec2<f32>(0.5));
    let sample_coord = vec2<i32>(top_left + rounded);

    let sample = sample_pixel(sample_coord);
    if !(tex_pos.x < 0.0 || tex_pos.y < 0.0 || tex_pos.x > 1.0 || tex_pos.y > 1.0) {
        return sample;
    } else {
        return image_display.clear_colour;
    }
}

fn billinear_filtering(pos: vec2<f32>) -> vec4<f32> {
    let tex_pos = screen_pos_to_tex_coord(pos);

    let real_step = 1.0 / tex_size();

    let top_left = floor(tex_pos / real_step) * real_step;
    let top_right = top_left + vec2<f32>(real_step.x, 0.0);
    let bottom_left = top_left + vec2<f32>(0.0, real_step.y);
    let bottom_right = top_left + real_step;

    let top_left_sample = sample(top_left);
    let top_right_sample = sample(top_right);
    let bottom_left_sample = sample(bottom_left);
    let bottom_right_sample = sample(bottom_right);

    let top_middle = ((top_right.x - tex_pos.x) / (top_right.x - top_left.x)) * top_left_sample +
                     ((tex_pos.x - top_left.x) / (top_right.x - top_left.x)) * top_right_sample;

    let bottom_middle = ((bottom_right.x - tex_pos.x) / (bottom_right.x - bottom_left.x)) * bottom_left_sample +
                        ((tex_pos.x - bottom_left.x) / (bottom_right.x - bottom_left.x)) * bottom_right_sample;

    let middle_middle = ((bottom_left.y - tex_pos.y) / (bottom_left.y - top_left.y)) * top_middle +
                        ((tex_pos.y - top_left.y) / (bottom_left.y - top_left.y)) * bottom_middle;

    if !(tex_pos.x < 0.0 || tex_pos.y < 0.0 || tex_pos.x > 1.0 || tex_pos.y > 1.0) {
        return middle_middle;
    } else {
        return image_display.clear_colour;
    }
}

fn screen_pos_to_tex_coord(pos: vec2<f32>) -> vec2<f32> {
    return ((pos.xy - image_display.pos + (tex_size() * image_display.scale / 2.0)) / image_display.scale) / tex_size();
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

fn sample_pixel(pos : vec2<i32>) -> vec4<f32> {
    let transformed = (vec2<f32>(pos) + vec2<f32>(0.5)) / tex_size();
    return textureSample(t_diffuse, s_diffuse, transformed);
}

fn sample(pos : vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, pos);
}

// fn sample_pixel(pixel)

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    switch image_display.scaling_mode {
        case 0u: {
            return gamma_correction(nearest_neighbour(in.clip_position.xy));
        }
        case 1u: {
            return gamma_correction(billinear_filtering(in.clip_position.xy));
        }
        default: {
            return image_display.clear_colour;
        }
    }
}