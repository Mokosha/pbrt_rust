use geometry::point::Point;
use geometry::vector::Vector;
use utils::Clamp;
use utils::Lerp;

const NOISE_PERM_SIZE: usize = 256;
const NOISE_PERM: [usize; 2 * NOISE_PERM_SIZE] = [
    151, 160, 137,  91,  90,  15, 131,  13, 201,  95,  96,
     53, 194, 233,   7, 225, 140,  36, 103,  30,  69, 142,
      8,  99,  37, 240,  21,  10,  23, 190,   6, 148, 247,
    120, 234,  75,   0,  26, 197,  62,  94, 252, 219, 203,
    117,  35,  11,  32,  57, 177,  33,  88, 237, 149,  56,
     87, 174,  20, 125, 136, 171, 168,  68, 175,  74, 165,
     71, 134, 139,  48,  27, 166,  77, 146, 158, 231,  83,
    111, 229, 122,  60, 211, 133, 230, 220, 105,  92,  41,
     55,  46, 245,  40, 244, 102, 143,  54,  65,  25,  63,
    161,   1, 216,  80,  73, 209,  76, 132, 187, 208,  89,
     18, 169, 200, 196, 135, 130, 116, 188, 159,  86, 164,
    100, 109, 198, 173, 186,   3,  64,  52, 217, 226, 250,
    124, 123,   5, 202,  38, 147, 118, 126, 255,  82,  85,
    212, 207, 206,  59, 227,  47,  16,  58,  17, 182, 189,
     28,  42, 223, 183, 170, 213, 119, 248, 152,   2,  44,
    154, 163,  70, 221, 153, 101, 155, 167,  43, 172,   9,
    129,  22,  39, 253,  19,  98, 108, 110,  79, 113, 224,
    232, 178, 185, 112, 104, 218, 246,  97, 228, 251,  34,
    242, 193, 238, 210, 144,  12, 191, 179, 162, 241,  81,
     51, 145, 235, 249,  14, 239, 107,  49, 192, 214,  31,
    181, 199, 106, 157, 184,  84, 204, 176, 115, 121,  50,
     45, 127,   4, 150, 254, 138, 236, 205,  93, 222, 114,
     67,  29,  24,  72, 243, 141, 128, 195,  78,  66, 215,
     61, 156, 180,
    151, 160, 137,  91,  90,  15, 131,  13, 201,  95,  96,
     53, 194, 233,   7, 225, 140,  36, 103,  30,  69, 142,
      8,  99,  37, 240,  21,  10,  23, 190,   6, 148, 247,
    120, 234,  75,   0,  26, 197,  62,  94, 252, 219, 203,
    117,  35,  11,  32,  57, 177,  33,  88, 237, 149,  56,
     87, 174,  20, 125, 136, 171, 168,  68, 175,  74, 165,
     71, 134, 139,  48,  27, 166,  77, 146, 158, 231,  83,
    111, 229, 122,  60, 211, 133, 230, 220, 105,  92,  41,
     55,  46, 245,  40, 244, 102, 143,  54,  65,  25,  63,
    161,   1, 216,  80,  73, 209,  76, 132, 187, 208,  89,
     18, 169, 200, 196, 135, 130, 116, 188, 159,  86, 164,
    100, 109, 198, 173, 186,   3,  64,  52, 217, 226, 250,
    124, 123,   5, 202,  38, 147, 118, 126, 255,  82,  85,
    212, 207, 206,  59, 227,  47,  16,  58,  17, 182, 189,
     28,  42, 223, 183, 170, 213, 119, 248, 152,   2,  44,
    154, 163,  70, 221, 153, 101, 155, 167,  43, 172,   9,
    129,  22,  39, 253,  19,  98, 108, 110,  79, 113, 224,
    232, 178, 185, 112, 104, 218, 246,  97, 228, 251,  34,
    242, 193, 238, 210, 144,  12, 191, 179, 162, 241,  81,
     51, 145, 235, 249,  14, 239, 107,  49, 192, 214,  31,
    181, 199, 106, 157, 184,  84, 204, 176, 115, 121,  50,
     45, 127,   4, 150, 254, 138, 236, 205,  93, 222, 114,
     67,  29,  24,  72, 243, 141, 128, 195,  78,  66, 215,
     61, 156, 180,
];

fn grad(x: usize, y: usize, z: usize, dx: f32, dy: f32, dz: f32) -> f32 {
    let h = NOISE_PERM[NOISE_PERM[NOISE_PERM[x] + y] + z] & 15;
    let u = if h < 8 || h == 12 || h == 13 { dx } else { dy };
    let v = if h < 4 || h == 12 || h == 13 { dy } else { dz };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

fn noise_weight(t: f32) -> f32 {
    let t3 = t * t * t;
    let t4 = t3 * t;
    6.0 * t4 * t - 15.0 * t4 + 10.0 * t3
}

pub fn noise(x: f32, y: f32, z: f32) -> f32 {
    // Compute noise cell coordinates and offsets
    let noise_mask = (NOISE_PERM_SIZE - 1) as i32;
    let ix = ((x.floor() as i32) & noise_mask) as usize;
    let iy = ((y.floor() as i32) & noise_mask) as usize;
    let iz = ((z.floor() as i32) & noise_mask) as usize;

    let dx = x - x.floor();
    let dy = y - y.floor();
    let dz = z - z.floor();

    // Compute gradient weights
    let w000 = grad(ix    , iy    , iz    , dx      , dy      , dz      );
    let w100 = grad(ix + 1, iy    , iz    , dx - 1.0, dy      , dz      );
    let w010 = grad(ix    , iy + 1, iz    , dx      , dy - 1.0, dz      );
    let w110 = grad(ix + 1, iy + 1, iz    , dx - 1.0, dy - 1.0, dz      );
    let w001 = grad(ix    , iy    , iz + 1, dx      , dy      , dz - 1.0);
    let w101 = grad(ix + 1, iy    , iz + 1, dx - 1.0, dy      , dz - 1.0);
    let w011 = grad(ix    , iy + 1, iz + 1, dx      , dy - 1.0, dz - 1.0);
    let w111 = grad(ix + 1, iy + 1, iz + 1, dx - 1.0, dy - 1.0, dz - 1.0);

    // Compute trilinear interpolation weights
    let wx = noise_weight(dx);
    let wy = noise_weight(dy);
    let wz = noise_weight(dz);
    let x00 = w000.lerp_with(w100, wx);
    let x10 = w010.lerp_with(w110, wx);
    let x01 = w001.lerp_with(w101, wx);
    let x11 = w011.lerp_with(w111, wx);
    let y0 = x00.lerp_with(x10, wy);
    let y1 = x01.lerp_with(x11, wy);
    y0.lerp_with(y1, wz)
}

pub fn noise_at(p: &Point) -> f32 { noise(p.x, p.y, p.z) }

fn smoothstep(min: f32, max: f32, value: f32) -> f32 {
    let v = ((value - min) / (max - min)).clamp(0.0, 1.0);
    v * v * (-2.0 * v + 3.0)
}

pub fn fbm(p: &Point, dpdx: &Vector, dpdy: &Vector,
           omega: f32, max_octaves: i32) -> f32 {
    // Compute number of octaves for antialiased FBm
    let s2 = dpdx.length_squared().max(dpdy.length_squared());
    let foctaves = (max_octaves as f32).min(1.0 - 0.5 * s2.log2());
    let octaves = foctaves.floor() as i32;

    // Compute sum of octaves of noise for FBm
    let (sum, lambda, o) = (0..octaves).fold(
        (0.0, 1.0, 1.0), |(acc, lambda, o), i| {
            (acc + o * noise_at(&(lambda * p)), lambda * 1.99, o * omega)
        });
    let partial_octave = foctaves - foctaves.floor();
    o * smoothstep(0.3, 0.7, partial_octave) * noise_at(&(lambda * p))
}
