use camera::CameraSample;
use filter::Filter;
use spectrum::Spectrum;

const FILTER_TABLE_DIM: usize = 16;
const FILTER_TABLE_SIZE: usize = FILTER_TABLE_DIM * FILTER_TABLE_DIM;

#[derive(Debug, Clone, PartialEq)]
pub struct Pixel {
    xyz: [f32; 3],
    splat_xyz: [f32; 3],
    weight_sum: f32,
    _pad: f32
}

impl Pixel {
    fn new() -> Pixel {
        Pixel {
            xyz: [0.0, 0.0, 0.0],
            weight_sum: 0.0,
            splat_xyz: [0.0, 0.0, 0.0],
            _pad: 0.0
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilmTy {
    Image {
        filter: Filter,
        crop_window: [f32; 4],
        filename: String,
        x_pixel_start: i32,
        y_pixel_start: i32,
        x_pixel_count: usize,
        y_pixel_count: usize,
        pixels: Vec<Pixel>,  // !SPEED! Use a z-ordering here
        filter_table: Vec<f32>
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Film {
    x_res: usize,
    y_res: usize,
    ty: FilmTy
}

impl Film {
    pub fn image(xres: usize, yres: usize, filter: Filter,
                 crop: [f32; 4], filename: String,
                 open_window: bool) -> Film {
        // Compute film image extent
        let x_start = ((xres as f32) * crop[0]).ceil() as i32;
        let x_count = ::std::cmp::max((((xres as f32) * crop[1]).ceil() as i32) - x_start, 1) as usize;
        let y_start = ((yres as f32) * crop[2]).ceil() as i32;
        let y_count = ::std::cmp::max((((yres as f32) * crop[3]).ceil() as i32) - y_start, 1) as usize;

        debug_assert!(x_count > 0);
        debug_assert!(y_count > 0);

        // Precompute filter weight table
        let mut ft = vec![0.0; FILTER_TABLE_SIZE];
        for y in 0..FILTER_TABLE_DIM {
            let fy = ((y as f32) + 0.5) * filter.y_width() / (FILTER_TABLE_DIM as f32);
            for x in 0..FILTER_TABLE_DIM {
                let fx = ((x as f32) + 0.5) * filter.x_width() / (FILTER_TABLE_DIM as f32);
                ft[y * FILTER_TABLE_DIM + x] = filter.evaluate(fx, fy);
            }
        }

        // Possibly open window for image display
        if open_window {
            unimplemented!()
        }

        Film {
            x_res: xres,
            y_res: yres,
            ty: FilmTy::Image {
                filter: filter,
                crop_window: crop,
                filename: filename,
                x_pixel_start: x_start,
                x_pixel_count: x_count,
                y_pixel_start: y_start,
                y_pixel_count: y_count,

                // Allocate film image storage
                pixels: vec![Pixel::new(); x_count * y_count],

                filter_table: ft
            }
        }
    }

    pub fn x_res(&self) -> usize { self.x_res }
    pub fn y_res(&self) -> usize { self.y_res }

    pub fn num_pixels(&self) -> usize { self.x_res * self.y_res }
    pub fn add_sample(&mut self, sample: &CameraSample, ls: &Spectrum) {
    }

    pub fn splat(&mut self, sample: &CameraSample, ls: &Spectrum) {
    }

    pub fn get_sample_extent(&self) -> (i32, i32, i32, i32) {
        (0, 0, 0, 0)
    }

    pub fn get_pixel_extent(&self) -> (i32, i32, i32, i32) {
        (0, 0, 0, 0)
    }

    pub fn update_display(&mut self, x0: i32, y0: i32, x1: i32, y1: i32,
                          splat_scale: f32) {
    }

    pub fn write_image(&mut self, splat_scale: f32) {
    }
}
