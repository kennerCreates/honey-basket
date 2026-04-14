@group(0) @binding(0) var input: texture_2d<f32>;
@group(0) @binding(1) var output: texture_storage_2d<rgba8unorm, write>;

  @compute @workgroup_size(8, 8)
  fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var empty_color = vec4(0.0, 0.0, 0.0, 1.0); //black
    var head_color = vec4(0.6, 0.6, 0.0, 1.0); //yellow
    var tail_color = vec4(0.3, 0.3, 1.0, 1.0); //blue
    var wire_color = vec4(1.0, 0.5, 0.0, 1.0); //orange
    var head_count = 0;
    var state: f32 = textureLoad(input, vec2<i32>(id.xy), 0).r;
    var offsets: array<vec2<i32>, 8> = array<vec2<i32>, 8>(
      vec2(-1, -1),
      vec2( 0, -1),
      vec2( 1, -1),
      vec2(-1,  0),
      vec2( 1,  0),
      vec2(-1,  1),
      vec2( 0,  1),
      vec2( 1,  1),
    );
    var input_size = vec2<i32>(textureDimensions(input));
    for (var i = 0; i < 8; i++) {
      let wrapped_coord = (vec2<i32>(id.xy) + offsets[i] + input_size) % input_size;
      let neighbor_r = textureLoad(input, wrapped_coord,0).r;
      if (neighbor_r > 0.5 && neighbor_r < 1.0) {
        head_count += 1;
      }
    }

    if (state < 0.25) {
      textureStore(output, id.xy, empty_color);
    } else if (state < 0.5) {
      textureStore(output, id.xy, wire_color);
    } else if (state < 0.75) {
      textureStore(output, id.xy, tail_color);
    } else if (head_count == 2 || head_count == 1) {
      textureStore(output, id.xy, head_color);
    } else {
      textureStore(output, id.xy, wire_color);
    }


  }
