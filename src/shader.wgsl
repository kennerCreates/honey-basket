@group(0) @binding(0) var input: texture_2d<f32>;
@group(0) @binding(1) var output: texture_storage_2d<rgba8unorm, write>;

  @compute @workgroup_size(8, 8)
  fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var dead_color = vec4(0.0, 0.0, 0.0, 1.0);
    var alive_color = vec4(1.0, 1.0, 0.0, 1.0);
    var alive_count = 0;
    var is_alive: bool = textureLoad(input, vec2<i32>(id.xy), 0).r > 0.5;
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

    for (var i = 0; i < 8; i++) {
      if (textureLoad(input, vec2<i32>(id.xy) + offsets[i], 0).r > 0.5) {
        alive_count += 1;
      }
    }

    if (alive_count < 2 || alive_count > 3) {
      textureStore(output, id.xy, dead_color);
    } else if (alive_count == 3) {
      textureStore(output, id.xy, alive_color);
    } else if (is_alive && alive_count == 2) {
      textureStore(output, id.xy, alive_color);
    } else {
      textureStore(output, id.xy, dead_color);
    }
  }
