// bindings
@group(0)@binding(0)
var t: texture_2d<f32>;

@group(0)@binding(1)
var s: sampler;

//vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

//vertex shader
@vertex
fn vs(@builtin(vertex_index) id: u32) -> VertexOutput {
    var output: VertexOutput;

    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );
    output.position = vec4(positions[id], 0.0, 1.0);
    output.uv = uvs[id];
    return output;
}

//fragment shader
@fragment
fn fs(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t, s, input.uv);
}
