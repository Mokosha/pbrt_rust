use diff_geom::DifferentialGeometry;
use texture::internal::TextureBase;
use texture::mapping3d::TextureMapping3D;

use texture::noise::fbm;
use texture::noise::turbulence;

#[derive(Debug)]
pub struct FBmTexture {
    omega: f32,
    octaves: i32,
    mapping: Box<TextureMapping3D>
}

impl FBmTexture {
    pub fn new(oct: i32, roughness: f32, map: Box<TextureMapping3D>) -> FBmTexture {
        FBmTexture { omega: roughness, octaves: oct, mapping: map }
    }
}

impl TextureBase<f32> for FBmTexture {
    fn eval(&self, dg: &DifferentialGeometry) -> f32 {
        let (p, dpdx, dpdy) = self.mapping.map(dg);
        fbm(&p, &dpdx, &dpdy, self.omega, self.octaves)
    }
}

#[derive(Debug)]
pub struct WrinkledTexture {
    omega: f32,
    octaves: i32,
    mapping: Box<TextureMapping3D>
}

impl WrinkledTexture {
    pub fn new(oct: i32, roughness: f32, map: Box<TextureMapping3D>)
               -> WrinkledTexture {
        WrinkledTexture { omega: roughness, octaves: oct, mapping: map }
    }
}

impl TextureBase<f32> for WrinkledTexture {
    fn eval(&self, dg: &DifferentialGeometry) -> f32 {
        let (p, dpdx, dpdy) = self.mapping.map(dg);
        turbulence(&p, &dpdx, &dpdy, self.omega, self.octaves)
    }
}
