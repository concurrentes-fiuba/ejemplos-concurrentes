@group(0) @binding(0) var<storage, read_write> result: array<u32>;

@compute @workgroup_size(32) fn computeSomething(
  @builtin(num_workgroups) num_workgroups: vec3<u32>,
  @builtin(global_invocation_id) global_invocation_id : vec3<u32>) {

    let xy = global_invocation_id.x + (global_invocation_id.y * num_workgroups.x * 32);
    let pixel = result[xy];
    let gray = u32(f32(pixel & 0xFF) * 0.299 + f32((pixel >> 8) & 0xFF) * 0.587 + f32((pixel >> 16) & 0xFF) * 0.114) & 0xFF;
    result[xy] = gray | gray << 8 | gray << 16;
}