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

fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

fn sample_pixel(pos : vec2<i32>) -> vec4<f32> {
    let clamped = min(max(pos, vec2<i32>(0)), vec2<i32>(tex_size()));
    let transformed = (vec2<f32>(clamped) + vec2<f32>(0.5)) / tex_size();
    return textureSample(t_diffuse, s_diffuse, transformed);
}

fn nearest_neighbour(tex_pos: vec2<f32>) -> vec4<f32> {
    let real_step = 1.0 / tex_size();

    let top_left = floor(tex_pos / real_step);
    let rounded = round((tex_pos % real_step) / real_step - vec2<f32>(0.5));
    let sample_coord = vec2<i32>(top_left + rounded);

    return sample_pixel(sample_coord);
}

fn billinear_filtering(tex_pos: vec2<f32>) -> vec4<f32> {
    let real_step = 1.0 / tex_size();

    let pixel_pos = tex_pos / real_step;

    let top_left = floor(tex_pos / real_step);
    let top_right = top_left + vec2<f32>(1.0, 0.0);
    let bottom_left = top_left + vec2<f32>(0.0, 1.0);
    let bottom_right = top_left + vec2<f32>(1.0);

    let top_left_sample = sample_pixel(vec2<i32>(top_left));
    let top_right_sample = sample_pixel(vec2<i32>(top_right));
    let bottom_left_sample = sample_pixel(vec2<i32>(bottom_left));
    let bottom_right_sample = sample_pixel(vec2<i32>(bottom_right));

    // let top_left = floor(tex_pos / real_step) * real_step;
    // let top_right = top_left + vec2<f32>(real_step.x, 0.0);
    // let bottom_left = top_left + vec2<f32>(0.0, real_step.y);
    // let bottom_right = top_left + real_step;

    // let top_left_sample = sample(top_left);
    // let top_right_sample = sample(top_right);
    // let bottom_left_sample = sample(bottom_left);
    // let bottom_right_sample = sample(bottom_right);

    let top_middle = ((top_right.x - pixel_pos.x) / (top_right.x - top_left.x)) * top_left_sample +
                     ((pixel_pos.x - top_left.x) / (top_right.x - top_left.x)) * top_right_sample;

    let bottom_middle = ((bottom_right.x - pixel_pos.x) / (bottom_right.x - bottom_left.x)) * bottom_left_sample +
                        ((pixel_pos.x - bottom_left.x) / (bottom_right.x - bottom_left.x)) * bottom_right_sample;

    let middle_middle = ((bottom_left.y - pixel_pos.y) / (bottom_left.y - top_left.y)) * top_middle +
                        ((pixel_pos.y - top_left.y) / (bottom_left.y - top_left.y)) * bottom_middle;

    return middle_middle;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let point = in.clip_position.xy / (tex_size() * image_display.scale);
    switch image_display.scaling_mode {
        case 0u: {
            return nearest_neighbour(point);
        }
        case 1u: {
            return billinear_filtering(point);
        }
        default: {
            discard;
        }
    }
}