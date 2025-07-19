@group(0) @binding(0)
var<storage, read> input: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

@compute @workgroup_size(64)
fn sha256(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    // Each thread processes one 512-bit (16 u32) block starting at index * 16
    let block_start = index * 16u;
    
    // Check for out-of-bounds
    if (block_start + 16u > arrayLength(&input)) {
        return;
    }
    
    // Load the input block for this thread
    var input_block: array<u32, 16>;
    for (var i = 0u; i < 16u; i = i + 1u) {
        input_block[i] = input[block_start + i];
    }
    
    // Compute the SHA-256 hash
    let hash = compute_sha256(input_block);
    
    // Write the 256-bit (8 u32) hash to the output buffer
    let output_start = index * 8u;
    if (output_start + 8u <= arrayLength(&output)) {
        for (var i = 0u; i < 8u; i = i + 1u) {
            output[output_start + i] = hash[i];
        }
    }
}