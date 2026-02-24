@group(0) @binding(0) var<storage, read_write> result: array<vec3<u32>>;

const workgroup_size = vec3(8, 2, 2);

// workgroup_size(32)
@compute @workgroup_size(8, 2, 2) fn computeSomething(
  @builtin(workgroup_id) workgroup_id : vec3<u32>,
  @builtin(num_workgroups) num_workgroups: vec3<u32>,
  @builtin(local_invocation_id) local_invocation_id : vec3<u32>,
  @builtin(global_invocation_id) global_invocation_id : vec3<u32>) {

    let total_size = num_workgroups * workgroup_size;

    // Linearize 3D global_invocation_id to 1D
    let linear_id = global_invocation_id.x +
                    (global_invocation_id.y * total_size.x) +
                    (global_invocation_id.z * total_size.x * total_size.y);


    let result_position = linear_id * 3;

    result[result_position] = global_invocation_id;
    result[result_position + 1] = workgroup_id;
    result[result_position + 2] = local_invocation_id;
}