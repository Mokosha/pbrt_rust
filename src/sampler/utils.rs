fn van_der_corput(_n: u32, scramble: u32) -> f32 {
    let mut n = _n;

    // Reverse bits of n
    n = (n << 16) | (n >> 16);
    n = ((n & 0x00ff00ff) << 8) | ((n & 0xff00ff00) >> 8);
    n = ((n & 0x0f0f0f0f) << 4) | ((n & 0xf0f0f0f0) >> 4);
    n = ((n & 0x33333333) << 2) | ((n & 0xCCCCCCCC) >> 2);
    n = ((n & 0x55555555) << 2) | ((n & 0xAAAAAAAA) >> 2);
    
    n ^= scramble;
    ((((n >> 8) & 0xffffff) as f64) / ((1 << 24) as f64)) as f32
}

fn sobol2(_n: u32, scramble: u32) -> f32 {
    let mut s = scramble;
    let mut n = _n;
    let mut v: u32 = 1 << 31;
    while n != 0 {
        if (n & 0x1) == 0 {
            s ^= v;
        }
        v ^= (v >> 1);
        n >>= 1;
    }

    ((((s >> 8) & 0xFFFFFF) as f64) / ((1 << 24) as f64)) as f32
}

pub fn sample02(n: u32, scramble: [u32; 2]) -> (f32, f32) {
    (van_der_corput(n, scramble[0]), sobol2(n, scramble[1]))
}
