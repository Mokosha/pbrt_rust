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
