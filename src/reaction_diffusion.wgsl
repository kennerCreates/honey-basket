@group(0) @binding(0) var input: texture_2d<f32>;
@group(0) @binding(1) var output: texture_storage_2d<rgba8unorm, write>;

const Du = 1.0;
const Dv = 0.5;
const F = 0.055;
const K = 0.062;
const dt = 1.0;


  @compute @workgroup_size(8, 8)
  fn main(@builtin(global_invocation_id) id: vec3<u32>) {

    var input_size = vec2<i32>(textureDimensions(input));
    var offsets: array<vec2<i32>, 9> = array<vec2<i32>, 9>(
        vec2( 0,  0),
        vec2(-1, -1),
        vec2( 0, -1),
        vec2( 1, -1),
        vec2(-1,  0),
        vec2( 1,  0),
        vec2(-1,  1),
        vec2( 0,  1),
        vec2( 1,  1),
    );
    var weights: array<f32, 9> = array<f32, 9>(
        -1.0,
        0.05,
        0.2,
        0.05,
        0.2,
        0.2,
        0.05,
        0.2,
        0.05,
    );
    var sum_u: f32 = 0.0;
    var sum_v: f32 = 0.0;
    var center: vec4<f32>;
    for (var i = 0; i < 9; i++) {
        let neighbor = textureLoad(input, ((vec2<i32>(id.xy) + offsets[i] + input_size) % input_size), 0);
        if (i == 0) {
            center = neighbor;
        }
        sum_u += neighbor.r * weights[i];
        sum_v += neighbor.g * weights[i];
    }
    let reaction_rate = center.r * center.g * center.g;
    var du = Du * sum_u - reaction_rate + F * (1.0 - center.r);
    var dv = Dv * sum_v + reaction_rate - (F + K) * center.g;
    var new_u = center.r + dt * du;
    var new_v = center.g + dt * dv;
    textureStore(output, id.xy, vec4(new_u, new_v, 0.0, 1.0));
  }
