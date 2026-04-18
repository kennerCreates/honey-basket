// bindings
@group(0)@binding(0)
var t: texture_2d<f32>;

@group(0)@binding(1)
var s: sampler;

@group(0)@binding(2)
var<uniform> u: Uniforms;
//vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Uniforms {
    width: f32,
    height: f32,
    sim_width: f32,
    sim_height: f32,
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
    var sim_aspect: f32 = u.sim_width / u.sim_height;
    var widget_aspect: f32 = u.width / u.height;
    var scale_x: f32 = min(1.0, sim_aspect / widget_aspect);
    var scale_y: f32 = min(1.0, widget_aspect / sim_aspect);
    var remapped_uv = (input.uv - 0.5) / vec2(scale_x, scale_y) + 0.5;
    if (any(remapped_uv < vec2<f32>(0.0)) || any(remapped_uv > vec2<f32>(1.0))) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    return textureSample(t, s, remapped_uv);
}
