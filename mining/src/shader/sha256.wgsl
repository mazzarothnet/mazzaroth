// SHA-256 constants (first 32 bits of the fractional parts of the cube roots of the first 64 primes)
fn k(i: u32) -> u32 {
    let K: array<u32, 64> = array<u32, 64>(
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    );
    return K[i];
}

// SHA-256 initial hash values (first 32 bits of the fractional parts of the square roots of the first 8 primes)
fn h_init(i: u32) -> u32 {
    let H: array<u32, 8> = array<u32, 8>(
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    );
    return H[i];
}

// SHA-256 bitwise functions
fn ch(x: u32, y: u32, z: u32) -> u32 {
    return (x & y) ^ (~x & z);
}

fn maj(x: u32, y: u32, z: u32) -> u32 {
    return (x & y) ^ (x & z) ^ (y & z);
}

fn sigma0(x: u32) -> u32 {
    return (x >> 2u | x << 30u) ^ (x >> 13u | x << 19u) ^ (x >> 22u | x << 10u);
}

fn sigma1(x: u32) -> u32 {
    return (x >> 6u | x << 26u) ^ (x >> 11u | x << 21u) ^ (x >> 25u | x << 7u);
}

fn little_sigma0(x: u32) -> u32 {
    return (x >> 7u | x << 25u) ^ (x >> 18u | x << 14u) ^ (x >> 3u);
}

fn little_sigma1(x: u32) -> u32 {
    return (x >> 17u | x << 15u) ^ (x >> 19u | x << 13u) ^ (x >> 10u);
}

fn compute_sha256(input_block: array<u32, 16>) -> array<u32, 8> {
    // Initialize hash values
    var h0 = h_init(0u);
    var h1 = h_init(1u);
    var h2 = h_init(2u);
    var h3 = h_init(3u);
    var h4 = h_init(4u);
    var h5 = h_init(5u);
    var h6 = h_init(6u);
    var h7 = h_init(7u);

    // Message schedule array
    var w: array<u32, 64>;

    // Load the 512-bit block (16 u32 words)
    for (var i = 0u; i < 16u; i = i + 1u) {
        w[i] = input_block[i];
    }

    // Prepare message schedule
    for (var i = 16u; i < 64u; i = i + 1u) {
        w[i] = little_sigma1(w[i - 2u]) + w[i - 7u] + little_sigma0(w[i - 15u]) + w[i - 16u];
    }

    // Initialize working variables
    var a = h0;
    var b = h1;
    var c = h2;
    var d = h3;
    var e = h4;
    var f = h5;
    var g = h6;
    var h = h7;

    // Compression loop
    for (var i = 0u; i < 64u; i = i + 1u) {
        let t1 = h + sigma1(e) + ch(e, f, g) + k(i) + w[i];
        let t2 = sigma0(a) + maj(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    // Update hash values
    h0 = h0 + a;
    h1 = h1 + b;
    h2 = h2 + c;
    h3 = h3 + d;
    h4 = h4 + e;
    h5 = h5 + f;
    h6 = h6 + g;
    h7 = h7 + h;

    return array<u32, 8>(h0, h1, h2, h3, h4, h5, h6, h7);
}
