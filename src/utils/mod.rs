pub mod kdtree;

pub trait Lerp<F = Self> {
    fn lerp(&self, b: &Self, t: F) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, b: &f32, t: f32) -> f32 {
        self * (1f32 - t) + b * t
    }
}

impl Lerp for f64 {
    fn lerp(&self, b: &f64, t: f64) -> f64 {
        self * (1f64 - t) + b * t
    }
}

pub trait Clamp<T = Self> {
    fn clamp(self, a: T, b: T) -> Self;
}

// These need no actual implementation since we can just use the default...
fn clamp_num<T: Copy+PartialOrd>(x: T, a: T, b: T) -> T {
    if x.lt(&a) { a } else if x.gt(&b) { b } else { x }
}

impl Clamp for f32 { fn clamp(self, a: f32, b: f32) -> f32 { clamp_num(self, a, b) } }
impl Clamp for f64 { fn clamp(self, a: f64, b: f64) -> f64 { clamp_num(self, a, b) } }
impl Clamp for i8  { fn clamp(self, a: i8, b: i8) -> i8 { clamp_num(self, a, b) } }
impl Clamp for i16 { fn clamp(self, a: i16, b: i16) -> i16 { clamp_num(self, a, b) } }
impl Clamp for i32 { fn clamp(self, a: i32, b: i32) -> i32 { clamp_num(self, a, b) } }
impl Clamp for i64 { fn clamp(self, a: i64, b: i64) -> i64 { clamp_num(self, a, b) } }
impl Clamp for u8  { fn clamp(self, a: u8, b: u8) -> u8 { clamp_num(self, a, b) } }
impl Clamp for u16 { fn clamp(self, a: u16, b: u16) -> u16 { clamp_num(self, a, b) } }
impl Clamp for u32 { fn clamp(self, a: u32, b: u32) -> u32 { clamp_num(self, a, b) } }
impl Clamp for u64 { fn clamp(self, a: u64, b: u64) -> u64 { clamp_num(self, a, b) } }
impl Clamp for usize { fn clamp(self, a: usize, b: usize) -> usize { clamp_num(self, a, b) } }

pub trait Degrees {
    fn as_radians(self) -> Self;
    fn as_degrees(self) -> Self;
}

impl Degrees for f32 {
    fn as_radians(self) -> f32 {
        self * ::std::f32::consts::PI / 180f32
    }

    fn as_degrees(self) -> f32 {
        self * 180f32 / ::std::f32::consts::PI
    }
}

impl Degrees for f64 {
    fn as_radians(self) -> f64 {
        self * ::std::f64::consts::PI / 180f64
    }

    fn as_degrees(self) -> f64 {
        self * 180f64 / ::std::f64::consts::PI
    }
}

pub fn quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    // Find quadratic discriminant
    let descrim = b * b - 4f32 * a * c;
    if descrim < 0.0 {
        return None;
    } else if descrim.abs() < 1e-6 {
        if a == 0.0 {
            return None;
        } else {
            let t = -b / (2.0 * a);
            return Some((t, t));
        }
    }

    let root_descrim = descrim.sqrt();

    // Compute quadratic t values
    let q = {
        if b < 0.0 {
            -0.5f32 * (b - root_descrim)
        } else {
            -0.5f32 * (b + root_descrim)
        }
    };

    if a == 0.0 {
        return None;
    }

    // Pretty sure this can never happen
    assert!(q != 0.0);

    let t0 = q / a;
    let t1 = c / q;

    if t0 < t1 { Some((t0, t1)) } else { Some((t1, t0)) }
}

pub fn solve_linear_system_2x2(a: [[f32; 2]; 2], b: [f32; 2])
                               -> Option<(f32, f32)> {
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if det.abs() < 1e-10 {
        return None;
    }

    let inv_det = 1.0 / det;
    let x0 = (a[1][1] * b[0] - a[0][1] * b[1]) * inv_det;
    let x1 = (a[0][0] * b[1] - a[1][0] * b[0]) * inv_det;

    if x0.is_nan() || x1.is_nan() {
        None
    } else {
        Some((x0, x1))
    }
}

pub fn partition_by<T, F, B>(v: &mut [T], f: F)
    where F: Fn(&T) -> B, B: Copy+PartialOrd
{
    let nv = v.len();
    if nv < 3 {
        if nv == 2 {
            if f(&v[1]) < f(&v[0]) {
                v.swap(0, 1);
            }
        }
        return;
    }

    let pivot = {
        // Median of three...
        let fst = f(&v[0]);
        let mid = f(&v[nv / 2]);
        let lst = f(&v[nv - 1]);

        if fst < mid && mid < lst {
            mid
        } else if mid < fst && fst < lst {
            fst
        } else {
            lst
        }
    };

    let mut last_smaller = 0;
    let mut num_pivots = 0;
    for i in 0..nv {
        let bv = f(&v[i]);
        if bv < pivot {
            v.swap(last_smaller + num_pivots, i);
            v.swap(last_smaller + num_pivots, last_smaller);
            last_smaller += 1;
        } else if bv == pivot {
            v.swap(last_smaller + num_pivots, i);
            num_pivots += 1;
        }
    }
    let mut pivot_idx = last_smaller;

    // We can do this because if pivot_idx == 0, then all
    // of the values are larger than the pivot...
    pivot_idx = ::std::cmp::max(pivot_idx, 1);

    let (left, right) = v.split_at_mut(pivot_idx);

    debug_assert!(right.len() > 0);
    debug_assert!(left.len() > 0);

    if pivot_idx + num_pivots <= (nv / 2) {
        partition_by(right, f);
    } else if pivot_idx >= (nv / 2) {
        partition_by(left, f);
    }
}

fn get_num_subwindows_2d(count: usize, aspect: usize) -> (usize, usize) {
    let mut nx = 1;
    let mut ny = count;
    while (ny % 2) == 0 && 2*aspect*nx < ny {
        ny /= 2;
        nx *= 2;
    }

    (nx, ny)
}

// Aspect is ratio of x to y dimension
pub fn get_crop_window(num: usize, count: usize,
                       aspect: f32) -> (f32, f32, f32, f32) {
    let (nx, ny) = if aspect < 1.0 {
        // Y dimension is larger than X
        let inva = (1.0 / aspect) as usize;
        get_num_subwindows_2d(count, inva)
    } else {
        let (x, y) = get_num_subwindows_2d(count, aspect as usize);
        (y, x)
    };

    // Compute x and y pixel sample range for sub window
    let xo = num % nx;
    let yo = num / nx;

    let tx0 = (xo as f32) / (nx as f32);
    let tx1 = ((xo + 1) as f32) / (nx as f32);

    let ty0 = (yo as f32) / (ny as f32);
    let ty1 = ((yo + 1) as f32) / (ny as f32);

    (tx0, tx1, ty0, ty1)
}

pub fn modulo(a: i32, b: i32) -> i32 {
    let n = a / b;
    let x = a - n * b;
    if x < 0 { x + b } else { x }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32;
    use std::f64;

    #[test]
    fn it_can_lerp_floats() {
        assert_eq!(0f32.lerp(&1f32, 0f32), 0f32);
        assert_eq!(0f32.lerp(&1f32, 1f32), 1f32);
        assert_eq!(0f32.lerp(&1f32, 0.1f32), 0.1f32);

        assert_eq!(1f32.lerp(&0f32, 0f32), 1f32);
        assert_eq!(1f32.lerp(&0f32, 1f32), 0f32);
        assert_eq!(1f32.lerp(&0f32, 0.1f32), 0.9f32);

        assert_eq!(1f32.lerp(&1f32, 0f32), 1f32);
        assert_eq!(0f32.lerp(&0f32, 1f32), 0f32);

        assert_eq!(0.1f32.lerp(&0.2f32, 0.1f32), 0.11f32);
        assert_eq!(0.1f32.lerp(&0.2f32, 1f32), 0.2f32);

        assert!(f32::NAN.lerp(&0.2f32, 1f32).is_nan());
        assert!(0.1.lerp(&f32::NAN, 1f32).is_nan());
        assert!(0.1.lerp(&0.2f32, f32::NAN).is_nan());

        assert_eq!(0f64.lerp(&1f64, 0f64), 0f64);
        assert_eq!(0f64.lerp(&1f64, 1f64), 1f64);
        assert_eq!(0f64.lerp(&1f64, 0.1f64), 0.1f64);

        assert_eq!(1f64.lerp(&0f64, 0f64), 1f64);
        assert_eq!(1f64.lerp(&0f64, 1f64), 0f64);
        assert_eq!(1f64.lerp(&0f64, 0.1f64), 0.9f64);

        assert_eq!(1f64.lerp(&1f64, 0f64), 1f64);
        assert_eq!(0f64.lerp(&0f64, 1f64), 0f64);

        assert!((0.1f64.lerp(&0.2f64, 0.1f64) - 0.11f64).abs() < 1e-6);
        assert!((0.1f64.lerp(&0.2f64, 1f64) - 0.2f64).abs() < 1e-6);

        assert!(f64::NAN.lerp(&0.2f64, 1f64).is_nan());
        assert!(0.1.lerp(&f64::NAN, 1f64).is_nan());
        assert!(0.1.lerp(&0.2f64, f64::NAN).is_nan());
    }

    #[test]
    fn it_can_clamp_numbers() {
        assert_eq!(0f32.clamp(1f32, 2f32), 1f32);
        assert_eq!(1f32.clamp(1f32, 2f32), 1f32);
        assert_eq!(1.5f32.clamp(1f32, 2f32), 1.5f32);
        assert_eq!(2f32.clamp(1f32, 2f32), 2f32);
        assert_eq!(3f32.clamp(1f32, 2f32), 2f32);

        assert_eq!(0f32.clamp(1f32, f32::NAN), 1f32);
        assert_eq!(0f32.clamp(f32::NAN, 1f32), 0f32);
        assert!(f32::NAN.clamp(1f32, 2f32).is_nan()); // can't un-nan

        assert_eq!(0f64.clamp(1f64, 2f64), 1f64);
        assert_eq!(1f64.clamp(1f64, 2f64), 1f64);
        assert_eq!(1.5f64.clamp(1f64, 2f64), 1.5f64);
        assert_eq!(2f64.clamp(1f64, 2f64), 2f64);
        assert_eq!(3f64.clamp(1f64, 2f64), 2f64);

        assert_eq!(0f64.clamp(1f64, f64::NAN), 1f64);
        assert_eq!(0f64.clamp(f64::NAN, 1f64), 0f64);
        assert!(f64::NAN.clamp(1f64, 2f64).is_nan()); // can't un-nan

        assert_eq!(0i8.clamp(1i8, 2i8), 1i8);
        assert_eq!(1i8.clamp(1i8, 2i8), 1i8);
        assert_eq!(2i8.clamp(1i8, 3i8), 2i8);
        assert_eq!(2i8.clamp(1i8, 2i8), 2i8);
        assert_eq!(3i8.clamp(1i8, 2i8), 2i8);

        assert_eq!(0i16.clamp(1i16, 2i16), 1i16);
        assert_eq!(1i16.clamp(1i16, 2i16), 1i16);
        assert_eq!(2i16.clamp(1i16, 3i16), 2i16);
        assert_eq!(2i16.clamp(1i16, 2i16), 2i16);
        assert_eq!(3i16.clamp(1i16, 2i16), 2i16);

        assert_eq!(0i32.clamp(1i32, 2i32), 1i32);
        assert_eq!(1i32.clamp(1i32, 2i32), 1i32);
        assert_eq!(2i32.clamp(1i32, 3i32), 2i32);
        assert_eq!(2i32.clamp(1i32, 2i32), 2i32);
        assert_eq!(3i32.clamp(1i32, 2i32), 2i32);

        assert_eq!(0i64.clamp(1i64, 2i64), 1i64);
        assert_eq!(1i64.clamp(1i64, 2i64), 1i64);
        assert_eq!(2i64.clamp(1i64, 3i64), 2i64);
        assert_eq!(2i64.clamp(1i64, 2i64), 2i64);
        assert_eq!(3i64.clamp(1i64, 2i64), 2i64);

        assert_eq!(0u8.clamp(1u8, 2u8), 1u8);
        assert_eq!(1u8.clamp(1u8, 2u8), 1u8);
        assert_eq!(2u8.clamp(1u8, 3u8), 2u8);
        assert_eq!(2u8.clamp(1u8, 2u8), 2u8);
        assert_eq!(3u8.clamp(1u8, 2u8), 2u8);

        assert_eq!(0u16.clamp(1u16, 2u16), 1u16);
        assert_eq!(1u16.clamp(1u16, 2u16), 1u16);
        assert_eq!(2u16.clamp(1u16, 3u16), 2u16);
        assert_eq!(2u16.clamp(1u16, 2u16), 2u16);
        assert_eq!(3u16.clamp(1u16, 2u16), 2u16);

        assert_eq!(0u32.clamp(1u32, 2u32), 1u32);
        assert_eq!(1u32.clamp(1u32, 2u32), 1u32);
        assert_eq!(2u32.clamp(1u32, 3u32), 2u32);
        assert_eq!(2u32.clamp(1u32, 2u32), 2u32);
        assert_eq!(3u32.clamp(1u32, 2u32), 2u32);

        assert_eq!(0u64.clamp(1u64, 2u64), 1u64);
        assert_eq!(1u64.clamp(1u64, 2u64), 1u64);
        assert_eq!(2u64.clamp(1u64, 3u64), 2u64);
        assert_eq!(2u64.clamp(1u64, 2u64), 2u64);
        assert_eq!(3u64.clamp(1u64, 2u64), 2u64);
    }

    #[test]
    fn it_can_convert_degrees_to_radians_and_back() {
        {
            let is_close = |x: f32, y: f32| { (x - y).abs() < 1e-6 };
            assert!(is_close(2f32 * f32::consts::PI, 360f32.as_radians()));
            assert!(is_close(f32::consts::PI, 180f32.as_radians()));
            assert!(is_close(f32::consts::FRAC_PI_2, 90f32.as_radians()));
            assert!(is_close(f32::consts::FRAC_PI_4, 45f32.as_radians()));

            assert!(is_close((2f32 * f32::consts::PI).as_degrees(), 360f32));
            assert!(is_close((f32::consts::PI).as_degrees(), 180f32));
            assert!(is_close((f32::consts::FRAC_PI_2).as_degrees(), 90f32));
            assert!(is_close((f32::consts::FRAC_PI_4).as_degrees(), 45f32));
        }

        {
            let is_close = |x: f64, y: f64| { (x - y).abs() < 1e-6 };
            assert!(is_close(2f64 * f64::consts::PI, 360f64.as_radians()));
            assert!(is_close(f64::consts::PI, 180f64.as_radians()));
            assert!(is_close(f64::consts::FRAC_PI_2, 90f64.as_radians()));
            assert!(is_close(f64::consts::FRAC_PI_4, 45f64.as_radians()));

            assert!(is_close((2f64 * f64::consts::PI).as_degrees(), 360f64));
            assert!(is_close((f64::consts::PI).as_degrees(), 180f64));
            assert!(is_close((f64::consts::FRAC_PI_2).as_degrees(), 90f64));
            assert!(is_close((f64::consts::FRAC_PI_4).as_degrees(), 45f64));
        }
    }

    #[test]
    fn it_can_solve_quadratic_equations() {
        assert_eq!(None, quadratic(1.0, 0.0, 1.0));
        assert_eq!(Some((2.0, 2.0)), quadratic(-1.0, 4.0, -4.0));
        assert_eq!(Some((0.0, 0.0)), quadratic(1.0, 0.0, 0.0));

        // parabola: (x - 1)^2 - 1 == x^2 - 2x
        assert_eq!(Some((0.0, 2.0)), quadratic(1.0, -2.0, 0.0));

        // As a matter of fact, we can generalize it:
        for i in 2..200 {
            assert_eq!(Some((0.0, i as f32)), quadratic(1.0, -(i as f32), 0.0));
        }

        // Last one: -(x - 4)^2 + 3 == -x^2 + 8x - 13
        // (+/-)sqrt(3) + 4
        let t0 = 3f32.sqrt() + 4.0;
        let t1 = -(3f32.sqrt()) + 4.0;
        assert_eq!(Some((t1, t0)), quadratic(-1.0, 8.0, -13.0));

        // Make sure that we handle "bad" input, too...
        assert_eq!(quadratic(0.0, 1.0, 1.0), None);
        assert_eq!(quadratic(0.0, 0.0, 1.75), None);
    }

    #[test]
    fn it_can_solve_2x2_linear_systems() {
        // Let's try the obvious case
        assert_eq!(solve_linear_system_2x2([[1.0, 0.0], [0.0, 1.0]], [3.5, -4.2]),
                   Some((3.5, -4.2)));

        // Let's try the degenerate case...
        assert_eq!(solve_linear_system_2x2([[::std::f32::NAN, 0.0], [0.0, 1.0]], [3.5, -4.2]),
                   None);
        assert_eq!(solve_linear_system_2x2([[1.0, ::std::f32::NAN], [0.0, 1.0]], [3.5, -4.2]),
                   None);
        assert_eq!(solve_linear_system_2x2([[1.0, 0.0], [::std::f32::NAN, 1.0]], [3.5, -4.2]),
                   None);
        assert_eq!(solve_linear_system_2x2([[1.0, 0.0], [0.0, ::std::f32::NAN]], [3.5, -4.2]),
                   None);
        assert_eq!(solve_linear_system_2x2([[1.0, 0.0], [0.0, 1.0]], [::std::f32::NAN, -4.2]),
                   None);
        assert_eq!(solve_linear_system_2x2([[1.0, 0.0], [0.0, 1.0]], [3.5, ::std::f32::NAN]),
                   None);

        // Let's try another degenerate case...
        assert_eq!(solve_linear_system_2x2([[0.0, 0.0], [0.0, 0.0]], [3.5, -4.2]),
                   None);
        assert_eq!(solve_linear_system_2x2([[1.0, 2.0], [1.0, 2.0]], [3.5, -4.2]),
                   None);
        assert_eq!(solve_linear_system_2x2([[3.0, 2.0], [6.0, 4.0]], [3.5, -4.2]),
                   None);

        // Let's try a rotation
        let sqrt2_2 = 0.5 * 2f32.sqrt();
        let (ox, oy) = solve_linear_system_2x2([[sqrt2_2, -sqrt2_2],
                                                [sqrt2_2,  sqrt2_2]],
                                               [3.0, 1.0]).unwrap();
        assert!((ox - (2.0 * 2f32.sqrt())).abs() < 1e-6);
        assert!((oy + 2f32.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn it_can_partition_slices() {
        let mut xs = [2, 5, 6, 1, 0, 4, 3, 2, 5, 1, 3, 3, 5];
        partition_by(&mut xs, |x| *x);

        let mid = xs.len() / 2;
        for i in 0..mid {
            for j in mid..(xs.len()) {
                assert!(xs[i] <= xs[j]);
            }
        }

        let mut ys = [1, 2];
        partition_by(&mut ys, |x| *x);
        assert_eq!(ys, [1, 2]);

        ys = [2, 1];
        partition_by(&mut ys, |x| *x);
        assert_eq!(ys, [1, 2]);

        let mut zs = [1];
        partition_by(&mut zs, |x| *x);
        assert_eq!(zs, [1]);

        let mut ps = [1, 0, 0, 0, -1, 0];
        partition_by(&mut ps, |x| *x);

        let m = ps.len() / 2;
        for i in 0..m {
            for j in m..(ps.len()) {
                assert!(ps[i] <= ps[j]);
            }
        }

        let mut qs = [3, 1, 1];
        partition_by(&mut qs, |x| *x);
        assert_eq!(qs, [1, 1, 3]);

        qs = [3, 1, 2];
        partition_by(&mut qs, |x| *x);        
        assert_eq!(qs, [1, 2, 3]);

        qs = [3, 3, 1];
        partition_by(&mut qs, |x| *x);        
        assert_eq!(qs, [1, 3, 3]);

        qs = [2, 2, 2];
        partition_by(&mut qs, |x| *x);
        assert_eq!(qs, [2, 2, 2]);

        let mut rs = [-2, -4, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4];
        partition_by(&mut rs, |x| *x);

        let m2 = rs.len() / 2;
        for i in 0..m2 {
            for j in m2..(rs.len()) {
                assert!(rs[i] <= rs[j]);
            }
        }

        rs = [-2, -4, 2, 2, 2, 4, 4, 4, 4, 4, 4, -5];
        partition_by(&mut rs, |x| *x);

        let m3 = rs.len() / 2;
        for i in 0..m3 {
            for j in m3..(rs.len()) {
                assert!(rs[i] <= rs[j]);
            }
        }
    }

    #[test]
    fn it_can_partition_points() {
        let mut pts = vec![
            ::geometry::point::Point::new_with(1.0, 1.0, -1.0),
            ::geometry::point::Point::new_with(-2.0, 2.0, -2.0),
            ::geometry::point::Point::new_with(2.0, 2.0, -2.0)];
        partition_by(&mut pts, |p| p[0]);
        assert_eq!(pts[0][0], -2.0);
    }

    #[test]
    fn it_can_get_a_cropped_window() {
        assert_eq!(get_crop_window(0, 2, 2.0), (0.0, 0.5, 0.0, 1.0));
        assert_eq!(get_crop_window(1, 2, 2.0), (0.5, 1.0, 0.0, 1.0));
        assert_eq!(get_crop_window(0, 2, 0.5), (0.0, 1.0, 0.0, 0.5));
        assert_eq!(get_crop_window(1, 2, 0.5), (0.0, 1.0, 0.5, 1.0));

        assert_eq!(get_crop_window(0, 4, 1.0), (0.0, 0.5, 0.0, 0.5));
        assert_eq!(get_crop_window(1, 4, 1.0), (0.5, 1.0, 0.0, 0.5));
        assert_eq!(get_crop_window(2, 4, 1.0), (0.0, 0.5, 0.5, 1.0));
        assert_eq!(get_crop_window(3, 4, 1.0), (0.5, 1.0, 0.5, 1.0));
    }

    #[test]
    fn it_can_do_modulo() {
        assert_eq!(modulo(1, 2), 1);
        assert_eq!(modulo(0, 2), 0);
        assert_eq!(modulo(2, 2), 0);
        assert_eq!(modulo(4, 3), 1);
        assert_eq!(modulo(-1, 2), 1);
        assert_eq!(modulo(-1, 3), 2);
        assert_eq!(modulo(-4, 3), 2);
        assert_eq!(modulo(-5, 3), 1);
    }
}
