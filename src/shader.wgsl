@group(0) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;

  @compute @workgroup_size(8, 8)
  fn main(@builtin(global_invocation_id) id: vec3<u32>) {
      textureStore(output, id.xy, vec4(1.0, 1.0, 0.0, 1.0));
  }
