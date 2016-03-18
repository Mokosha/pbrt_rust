pub struct FilterBase {
    x_width: f32,
    y_width: f32,
    inv_x_width: f32,
    inv_y_width: f32
}

impl FilterBase {
    fn new(xw: f32, yw: f32) -> FilterBase {
        FilterBase {
            x_width: xw,
            y_width: yw,
            inv_x_width: 1.0 / xw,
            inv_y_width: 1.0 / yw
        }
    }
}

pub enum Filter {
    Mean(FilterBase),  // Also known as a box filter
}

impl Filter {
    pub fn mean(xw: f32, yw: f32) -> Filter {
        Filter::Mean(FilterBase::new(xw, yw))
    }

    pub fn evaluate(&self, x: f32, y: f32) -> f32 {
        match self {
            &Filter::Mean(_) => 1.0
        }
    }

    pub fn x_width(&self) -> f32 { self.base().x_width }
    pub fn y_width(&self) -> f32 { self.base().y_width }
    pub fn inv_x_width(&self) -> f32 { self.base().inv_x_width }
    pub fn inv_y_width(&self) -> f32 { self.base().inv_y_width }

    // Private fns
    fn base(&self) -> &FilterBase {
        match self {
            &Filter::Mean(ref b) => b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_evaluate_box_filters() {
        let xs = [0.0, 1.0, -1.0, 16.0, 0.001];
        let ys = [2.0, 0.0, -0.01, ::std::f32::consts::PI];

        let filter = Filter::mean(1.0, 1.0);

        for &x in xs.iter() {
            for &y in ys.iter() {
                assert_eq!(filter.evaluate(x, y), 1.0);
            }
        }
    }
}
