@group(0) @binding(0)
var<storage, read> input: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

@compute @workgroup_size(64)
fn mining(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let output_start = index * 4u;
    for (var i = 0u; i < 4u; i = i + 1u) {
        output[output_start + i] = 0;
    }
    // Load the input block for this thread
    var input_block: array<u32, 16>;
    for (var i = 0u; i < 16u; i = i + 1u) {
        input_block[i] = input[i];
    }

    var target_value: array<u32, 8>;
    for (var i = 0u; i < 8u; i = i + 1u) {
        target_value[i] = input[i + 16];
    }
    input_block[10] = index;
    for (var i = 1u; i < 65536u; i = i + 1u) {
        input_block[11] = i;
        let hash = compute_sha256(input_block);
        var is_ok = 0;
        for (var i = 0u; i < 8; i = i + 1u) {
            if hash[i] < target_value[i] {
                is_ok = 1;
                break;
            }
            if hash[i] > target_value[i] {
                break;
            }
        }
        if is_ok == 1 {
            for (var i = 0u; i < 4u; i = i + 1u) {
                output[output_start + i] = input_block[8 + i];
            }
            break;
        }
    }
}