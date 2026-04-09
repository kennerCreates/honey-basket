@group(0) @binding(0) var input: texture_2d<f32>;
@group(0) @binding(1) var output: texture_storage_2d<rgba8unorm, write>;

  @compute @workgroup_size(8, 8)
  fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var color = textureLoad(input, id.xy, 0);
    var size = textureDimensions(output);
    textureStore(output, id.xy, vec4(f32(id.x)/f32(size.x), 0.0, f32(id.y)/f32(size.y), 1.0));
  }
