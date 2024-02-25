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

fn sample(point : vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, (point + vec2<f32>(0.5)) / tex_size());
}

fn min_in_vec(colour : vec3<f32>) -> f32 {
    return min(colour.x, min(colour.y, colour.z));
}

fn max_in_vec(colour : vec3<f32>) -> f32 {
    return max(colour.x, max(colour.y, colour.z));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var mini = 1.0;
    var maxi = 0.0;

    let size = vec2<i32>(tex_size() / vec2<f32>(8.0));
    let multiplied = size * vec2<i32>(in.clip_position.xy);

    for (var row = multiplied.y; row < i32(multiplied.y + size.y); row += 1) {
        for (var col = multiplied.x; col < i32(multiplied.x + size.x); col += 1) {
            let s = sample(vec2<f32>(f32(col), f32(row))).xyz;
            mini = min(mini, min_in_vec(s));
            maxi = max(maxi, max_in_vec(s));
        }
    }
    return vec4<f32>(mini, maxi, 0.0, 1.0);
}