struct ViewportDimensions {
    dimensions: vec2<f32>,
}
@group(0) @binding(0)
var<uniform> dims: ViewportDimensions;

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
    return vec4<f32>(pos.x / dims.dimensions.x, pos.y / dims.dimensions.y, 0.0, 1.0);
}