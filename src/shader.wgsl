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

fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

fn nearest_neighbour(pos: vec2<f32>) -> vec4<f32> {
    let tex_pos = screen_pos_to_tex_coord(pos);
    let real_step = 1.0 / tex_size();

    let top_left = floor(tex_pos / real_step);
    let rounded = round((tex_pos % real_step) / real_step - vec2<f32>(0.5));
    let sample_coord = vec2<i32>(top_left + rounded);

    return sample_pixel(sample_coord);
}

fn billinear_filtering(pos: vec2<f32>) -> vec4<f32> {
    let tex_pos = screen_pos_to_tex_coord(pos);
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

fn cross_correlation(pos: vec2<f32>) -> vec4<f32> {
    let tex_pos = screen_pos_to_tex_coord(pos);

    var results = array<vec4<f32>, 25>();
    var first = true;
    var mini = vec4<f32>(0.0);
    var maxi = vec4<f32>(0.0);

    for (var row = -2; row < 3; row += 1) {
        for (var col = -2; col < 3; col += 1) {
            let i = (row + 2) * 5 + (col + 2);
            let sample_pos = pos + vec2<f32>(f32(row) / image_display.window_size.y, f32(col) / image_display.window_size.x);
            let result = nearest_neighbour(sample_pos) * f32(laplacian[i]);

            if first {
                mini = result;
                maxi = result;
                first = false;
            } else {
                mini = min(mini, result);
                maxi = max(maxi, result);
            }

            results[i] = result;
        }
    }

    // let min_overall = vec4<f32>(min(mini.x, min(mini.y, mini.z)));
    // let max_overall = vec4<f32>(max(maxi.x, max(maxi.y, maxi.z)));

    let min_overall = vec4<f32>(-1.0);
    let max_overall = vec4<f32>(100.0);

    var sum = vec4<f32>(0.0);
    for (var r = 0; r < 25; r += 1) {
        sum += (results[r] - min_overall) / (max_overall - min_overall);
    }

    return sum;
}

fn screen_pos_to_tex_coord(pos: vec2<f32>) -> vec2<f32> {
    return ((pos.xy - image_display.pos - image_display.window_size / 2.0 + (tex_size() * image_display.scale / 2.0)) / image_display.scale) / tex_size();
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
    let clamped = min(max(pos, vec2<i32>(0)), vec2<i32>(tex_size()));
    let transformed = (vec2<f32>(clamped) + vec2<f32>(0.5)) / tex_size();
    return textureSample(t_diffuse, s_diffuse, transformed);
}

fn sample(pos : vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, pos);
}

// fn sample_pixel(pixel)

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_pos = screen_pos_to_tex_coord(in.clip_position.xy);
    if tex_pos.x < 0.0 || tex_pos.y < 0.0 || tex_pos.x > 1.0 || tex_pos.y > 1.0 {
        discard;
    }

    if image_display.cross_correlation > 0u {
        return cross_correlation(in.clip_position.xy);
    } else {
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
}