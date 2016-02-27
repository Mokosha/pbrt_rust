use utils::Lerp;

#[derive(Debug, Clone)]
pub struct SamplerBase {
    pub x_pixel_start: i32,
    pub x_pixel_end: i32,
    pub y_pixel_start: i32,
    pub y_pixel_end: i32,
    pub samples_per_pixel: usize,
    pub shutter_open: f32,
    pub shutter_close: f32
}

impl SamplerBase {
    pub fn new(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
               spp: usize, sopen: f32, sclose: f32) -> SamplerBase {
        SamplerBase {
            x_pixel_start: x_start,
            x_pixel_end: x_end,
            y_pixel_start: y_start,
            y_pixel_end: y_end,
            samples_per_pixel: spp,
            shutter_open: sopen,
            shutter_close: sclose
        }
    }

    pub fn compute_sub_window(&self, num: usize,
                              count: usize) -> (i32, i32, i32, i32) {
        // Determine how many tiles to use in each dimension nx and ny
        let dx = (self.x_pixel_end - self.x_pixel_start) as usize;
        let dy = (self.y_pixel_end - self.y_pixel_start) as usize;

        let (nx, ny) = if dx > dy {
            let mut _nx = count;
            let mut _ny = 1;
            while (_nx % 2) == 0 && 2*dx*_ny < dy*_nx {
                _nx /= 2;
                _ny *= 2;
            }
            (_nx, _ny)
        } else {
            let mut _nx = 1;
            let mut _ny = count;
            while (_ny % 2) == 0 && 2*dy*_nx < dx*_ny {
                _nx *= 2;
                _ny /= 2;
            }
            (_nx, _ny)
        };

        // Compute x and y pixel sample range for sub window
        let xo = num % nx;
        let yo = num / nx;

        let tx0 = (xo as f32) / (nx as f32);
        let tx1 = ((xo + 1) as f32) / (nx as f32);

        let ty0 = (yo as f32) / (ny as f32);
        let ty1 = ((yo + 1) as f32) / (ny as f32);

        let psx = self.x_pixel_start as f32;
        let psy = self.y_pixel_start as f32;
        let pex = self.x_pixel_end as f32;
        let pey = self.y_pixel_end as f32;

        let new_x_start = psx.lerp(&pex, tx0) as i32;
        let new_x_end = psx.lerp(&pex, tx1) as i32;
        let new_y_start = psy.lerp(&pey, ty0) as i32;
        let new_y_end = psy.lerp(&pey, ty1) as i32;

        (new_x_start, new_x_end, new_y_start, new_y_end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_be_created() {
        let s = SamplerBase::new(0, 1, 0, 1, 2, 0.0, 1.0);

        assert_eq!(s.x_pixel_start, 0);
        assert_eq!(s.x_pixel_end, 1);
        assert_eq!(s.y_pixel_start, 0);
        assert_eq!(s.y_pixel_end, 1);
        assert_eq!(s.samples_per_pixel, 2);
        assert_eq!(s.shutter_open, 0.0);
        assert_eq!(s.shutter_close, 1.0);
    }

    #[test]
    fn its_base_can_tile_windows() {
        let mut s = SamplerBase::new(0, 10, 0, 2, 2, 0.0, 1.0);

        assert_eq!(s.compute_sub_window(0, 20), (0, 1, 0, 1));
        assert_eq!(s.compute_sub_window(9, 20), (9, 10, 0, 1));
        assert_eq!(s.compute_sub_window(10, 20), (0, 1, 1, 2));
        assert_eq!(s.compute_sub_window(19, 20), (9, 10, 1, 2));

        assert_eq!(s.compute_sub_window(4, 5), (8, 10, 0, 2));
        assert_eq!(s.compute_sub_window(0, 1), (0, 10, 0, 2));

        s = SamplerBase::new(0, 2, 0, 10, 2, 0.0, 1.0);

        assert_eq!(s.compute_sub_window(0, 20), (0, 1, 0, 1));
        assert_eq!(s.compute_sub_window(9, 20), (1, 2, 4, 5));
        assert_eq!(s.compute_sub_window(10, 20), (0, 1, 5, 6));
        assert_eq!(s.compute_sub_window(19, 20), (1, 2, 9, 10));

        assert_eq!(s.compute_sub_window(4, 5), (0, 2, 8, 10));
        assert_eq!(s.compute_sub_window(0, 1), (0, 2, 0, 10));
    }
}
