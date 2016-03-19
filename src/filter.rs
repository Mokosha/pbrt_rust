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

pub enum FilterType {
    Mean,    // Also known as a box filter
    Triangle,
}

pub struct Filter {
    base: FilterBase,
    ty: FilterType
}

impl Filter {
    pub fn mean(xw: f32, yw: f32) -> Filter {
        Filter {
            base: FilterBase::new(xw, yw),
            ty: FilterType::Mean
        }
    }

    pub fn triangle(xw: f32, yw: f32) -> Filter {
        Filter {
            base: FilterBase::new(xw, yw),
            ty: FilterType::Triangle
        }
    }

    pub fn evaluate(&self, x: f32, y: f32) -> f32 {
        match &self.ty {
            &FilterType::Mean => 1.0,
            &FilterType::Triangle => {
                let dx = (self.x_width() - x.abs()) * self.inv_x_width();
                let dy = (self.y_width() - y.abs()) * self.inv_y_width();
                dx.max(0.0) * dy.max(0.0)
            }
        }
    }

    pub fn x_width(&self) -> f32 { self.base.x_width }
    pub fn y_width(&self) -> f32 { self.base.y_width }
    pub fn inv_x_width(&self) -> f32 { self.base.inv_x_width }
    pub fn inv_y_width(&self) -> f32 { self.base.inv_y_width }
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

    #[test]
    fn it_can_evaluate_triangle_filters() {
        let filter = Filter::triangle(2.0, 2.0);

        assert_eq!(filter.evaluate(1.0, 0.0), 0.5);
        assert_eq!(filter.evaluate(0.0, 0.0), 1.0);
        assert_eq!(filter.evaluate(20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(-20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, 20.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -20.0), 0.0);
        assert_eq!(filter.evaluate(1.0, 1.0), 0.25);
        assert_eq!(filter.evaluate(0.5, 0.5), 0.5625);
        assert_eq!(filter.evaluate(2.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.0), 0.0);
    }
}
