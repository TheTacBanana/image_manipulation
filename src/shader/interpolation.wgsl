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

// Get the size of the texture into the shader
fn tex_size() -> vec2<f32> {
    return vec2<f32>(textureDimensions(t_diffuse));
}

// Sample the texture at the pixel coordinate
fn sample_pixel(pos : vec2<i32>) -> vec4<f32> {
    let clamped = min(max(pos, vec2<i32>(0)), vec2<i32>(tex_size()));
    let transformed = (vec2<f32>(clamped) + vec2<f32>(0.5)) / tex_size();
    return textureSample(t_diffuse, s_diffuse, transformed);
}

// Perform nearest neighbour interpolation for the coordinate
fn nearest_neighbour(pos: vec2<f32>) -> vec4<f32> {
    let new_size = tex_size() * image_display.scale;
    let transformed = tex_size() * pos / new_size;
    let rounded = vec2<i32>(transformed);

    return sample_pixel(rounded);
}

// Perform billinear interpolation for the coordinate
fn billinear(pos: vec2<f32>) -> vec4<f32> {
    let new_size = tex_size() * image_display.scale;
    let transformed = (tex_size() * pos / new_size) - 0.5;

    // Get all texture coordinates
    let top_left = floor(transformed);
    let top_right = top_left + vec2<f32>(1.0, 0.0);
    let bottom_left = top_left + vec2<f32>(0.0, 1.0);
    let bottom_right = top_left + vec2<f32>(1.0);

    // Sample all coordinates
    let top_left_sample = sample_pixel(vec2<i32>(top_left));
    let top_right_sample = sample_pixel(vec2<i32>(top_right));
    let bottom_left_sample = sample_pixel(vec2<i32>(bottom_left));
    let bottom_right_sample = sample_pixel(vec2<i32>(bottom_right));

    // Intepolate between Top Left and Top Right
    let top_middle = ((top_right.x - transformed.x) / (top_right.x - top_left.x)) * top_left_sample +
                     ((transformed.x - top_left.x) / (top_right.x - top_left.x)) * top_right_sample;

    // Interpolate between Bottom Left and Bottom Right
    let bottom_middle = ((bottom_right.x - transformed.x) / (bottom_right.x - bottom_left.x)) * bottom_left_sample +
                        ((transformed.x - bottom_left.x) / (bottom_right.x - bottom_left.x)) * bottom_right_sample;

    // Interpolate between the previous two
    let middle_middle = ((bottom_left.y - transformed.y) / (bottom_left.y - top_left.y)) * top_middle +
                        ((transformed.y - top_left.y) / (bottom_left.y - top_left.y)) * bottom_middle;

    return middle_middle;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let point = in.clip_position.xy;
    switch image_display.scaling_mode {
        case 0u: {
            return nearest_neighbour(point);
        }
        case 1u: {
            return billinear(point);
        }
        default: {
            discard;
        }
    }
}