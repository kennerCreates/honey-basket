@group(0) @binding(0) var input: texture_2d<f32>;
@group(0) @binding(1) var output: texture_storage_2d<rgba8unorm, write>;

  @compute @workgroup_size(8, 8)
  fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var dead_color = vec4(0.0, 0.0, 0.0, 1.0);
    var alive_color = vec4(1.0, 1.0, 0.5, 1.0);
    var dying_color = vec4(0.5, 0.0, 0.5, 1.0);
    var alive_count = 0;
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
      if (textureLoad(input, ((vec2<i32>(id.xy) + offsets[i] + input_size) % input_size), 0).r > 0.75) {
        alive_count += 1;
      }
    }

    if (state > 0.75) {
      textureStore(output, id.xy, dying_color);
    } else if (state > 0.25) {
      textureStore(output, id.xy, dead_color);
    } else if alive_count == 2 {
      textureStore(output, id.xy, alive_color);
    } else {
      textureStore(output, id.xy, dead_color);
    }

  }
