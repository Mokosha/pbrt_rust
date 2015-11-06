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

pub trait Clamp : Copy+PartialOrd {
    fn clamp(self, a: Self, b: Self) -> Self {
        if self.lt(&a) { a } else if self.gt(&b) { b } else { self }
    }
}

impl Clamp for f32 { }
impl Clamp for f64 { }
impl Clamp for i8  { }
impl Clamp for i16 { }
impl Clamp for i32 { }
impl Clamp for i64 { }
impl Clamp for u8  { }
impl Clamp for u16 { }
impl Clamp for u32 { }
impl Clamp for u64 { }

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
}
