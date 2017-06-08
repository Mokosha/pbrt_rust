use utils::sinc_1d;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum FilterType {
    Mean,    // Also known as a box filter
    Triangle,
    Gaussian {
        alpha: f32,
        exp_x: f32,
        exp_y: f32
    },
    Mitchell {
        b: f32,
        c: f32
    },
    Lanczos(f32)
}

#[derive(Clone, Debug, PartialEq)]
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

    pub fn gaussian(xw: f32, yw: f32, a: f32) -> Filter {
        Filter {
            base: FilterBase::new(xw, yw),
            ty: FilterType::Gaussian {
                alpha: a,
                exp_x: (-a * xw * xw).exp(),
                exp_y: (-a * yw * yw).exp(),
            }
        }
    }

    pub fn mitchell(xw: f32, yw: f32, b: f32, c: f32) -> Filter {
        Filter {
            base: FilterBase::new(xw, yw),
            ty: FilterType::Mitchell {
                b: b,
                c: c
            }
        }
    }

    pub fn lanczos(xw: f32, yw: f32, tau: f32) -> Filter {
        Filter {
            base: FilterBase::new(xw, yw),
            ty: FilterType::Lanczos(tau)
        }
    }

    pub fn evaluate(&self, x: f32, y: f32) -> f32 {
        match &self.ty {
            &FilterType::Mean => 1.0,
            &FilterType::Triangle => {
                let dx = (self.x_width() - x.abs()) * self.inv_x_width();
                let dy = (self.y_width() - y.abs()) * self.inv_y_width();
                dx.max(0.0) * dy.max(0.0)
            },
            &FilterType::Gaussian { alpha, exp_x, exp_y } => {
                let gaussian = |v: f32, ex: f32| ((-alpha * v * v).exp() - ex).max(0.0);
                gaussian(x, exp_x) * gaussian(y, exp_y)
            },
            &FilterType::Mitchell { b, c } => {
                let mitchell = |v: f32| {
                    let t = (v * 2.0).abs();
                    (1.0 / 6.0) * if t >= 2.0 { 0.0 }
                    else if t > 1.0 {
                        (-b - 6.0*c) * t*t*t +
                            (6.0*b + 30.0*c) * t*t +
                            (-12.0*b - 48.0*c) * t +
                            (8.0*b + 24.0*c)
                    } else {
                        (12.0 - 9.0*b - 6.0*c) * t*t*t +
                            (-18.0 + 12.0*b + 6.0*c) * t*t +
                            (6.0 - 2.0*b)
                    }
                };

                let mx = mitchell(x * self.inv_x_width());
                let my = mitchell(y * self.inv_y_width());
                mx * my
            },
            &FilterType::Lanczos(tau) => {
                sinc_1d(x * self.inv_x_width(), tau) *
                    sinc_1d(y * self.inv_y_width(), tau)
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

    #[test]
    fn it_can_evaluate_gaussian_filters() {
        let filter = Filter::gaussian(2.0, 2.0, 1.0);

        assert_eq!(filter.evaluate(20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(-20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, 20.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -20.0), 0.0);
        assert_eq!(filter.evaluate(2.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.0), 0.0);

        assert!(filter.evaluate(1.0, 1.0) > 0.0);
        assert!(filter.evaluate(0.5, 0.0) > 0.0);
        assert!(filter.evaluate(1.0, -0.5) > 0.0);
        assert!(filter.evaluate(-1.0, -1.5) > 0.0);
        assert!(filter.evaluate(0.0, 0.0) > 0.9);
    }

    #[test]
    fn it_can_evaluate_lanczos_filters() {
        let filter = Filter::lanczos(2.0, 2.0, 1.0);

        assert_eq!(filter.evaluate(20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(-20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, 20.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -20.0), 0.0);
        assert_eq!(filter.evaluate(2.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.0), 0.0);

        assert!(filter.evaluate(0.0, 0.0) > 0.9);
    }

    #[test]
    fn it_can_evaluate_mitchell_filters() {
        let filter = Filter::mitchell(2.0, 2.0, 0.2, 0.4);

        assert_eq!(filter.evaluate(20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(-20.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, 20.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -20.0), 0.0);
        assert_eq!(filter.evaluate(2.0, 0.0), 0.0);
        assert_eq!(filter.evaluate(0.0, -2.0), 0.0);

        assert!(filter.evaluate(0.0, 0.0) > 0.8);
    }
}
