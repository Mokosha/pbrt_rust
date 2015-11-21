#[derive(Debug, PartialEq, Clone)]
pub struct Time {
    t: f32
}

impl ::std::convert::From<f32> for Time {
    fn from(f: f32) -> Time { Time { t: f } }
}

impl ::std::convert::From<Time> for f32 {
    fn from(time: Time) -> f32 { time.t }
}
