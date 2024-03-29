mod matte;
mod measured;
mod mix;
mod plastic;
mod subsurface;

use std::sync::Arc;

use bsdf::BSDF;
use bsdf::bssrdf::BSSRDF;
use diff_geom::DifferentialGeometry;
use geometry::vector::*;
use geometry::normal::*;
use spectrum::Spectrum;
use texture::{Texture, ScalarTextureReference, ColorTextureReference};

use material::matte::MatteMaterial;
use material::plastic::PlasticMaterial;
use material::measured::MeasuredMaterial;
use material::mix::MixMaterial;
use material::subsurface::SubsurfaceMaterial;

pub fn bump<Tex: Texture<f32>>(
    d: &Tex, dg_geom: &DifferentialGeometry,
    dg_shading: &DifferentialGeometry) -> DifferentialGeometry {
    // Compute offset positions and evaluate displacement texture
    let mut dg_eval = dg_shading.clone();

    // Shift dg_eval.du in the u direction
    let du = {
        let mdu = 0.5 * (dg_shading.dudx.abs() + dg_shading.dudy.abs());
        if mdu == 0.0 { 0.1 } else { mdu }
    };

    dg_eval.p = &dg_shading.p + du * &dg_shading.dpdu;
    dg_eval.u = dg_shading.u + du;
    dg_eval.nn = Normal::from(
        dg_shading.dpdu.cross_with(&dg_shading.dpdv) + du * &dg_shading.dndu)
        .normalize();

    let u_displace = d.evaluate(&dg_eval);

    // Shift dg_eval.dv in the v direction
    let dv = {
        let mdv = 0.5 * (dg_shading.dvdx.abs() + dg_shading.dvdy.abs());
        if mdv == 0.0 { 0.1 } else { mdv }
    };

    dg_eval.p = &dg_shading.p + dv * &dg_shading.dpdv;
    dg_eval.u = dg_shading.u;
    dg_eval.v = dg_shading.v + dv;
    dg_eval.nn = Normal::from(
        dg_shading.dpdu.cross_with(&dg_shading.dpdv) + dv * &dg_shading.dndv)
        .normalize();

    let v_displace = d.evaluate(&dg_eval);
    let displace = d.evaluate(dg_shading);

    // Compute bump mapped differential geometry
    let mut dg_bump = dg_shading.clone();
    dg_bump.dpdu = &dg_shading.dpdu
        + (u_displace - displace) / du * Vector::from(dg_shading.nn.clone())
        + displace * Vector::from(dg_shading.dndu.clone());
    dg_bump.dpdv = &dg_shading.dpdv
        + (v_displace - displace) / dv * Vector::from(dg_shading.nn.clone())
        + displace * Vector::from(dg_shading.dndv.clone());
    dg_bump.nn = Normal::from(dg_bump.dpdu.cross_with(&dg_bump.dpdv).normalize());
    if let Some(ref s) = dg_shading.shape {
        if s.reverse_orientation ^ s.transform_swaps_handedness {
            dg_bump.nn = Normal::new_with(-dg_bump.nn.x, -dg_bump.nn.y, -dg_bump.nn.z);
        }
    }

    // Orient shading normal to match geometric normal
    dg_bump.nn = dg_bump.nn.clone().face_forward(Vector::from(dg_geom.nn.clone()));
    dg_bump
}

#[derive(Clone, Debug)]
pub enum Material {
    Matte(MatteMaterial),
    Plastic(PlasticMaterial),
    Measured(MeasuredMaterial),
    Mixed(MixMaterial),
    Subsurface(SubsurfaceMaterial),
    Broken
}

impl Material {
    pub fn matte(kd: ColorTextureReference,
                 sig: ScalarTextureReference,
                 bump_map: Option<ScalarTextureReference>) -> Material {
        Material::Matte(MatteMaterial::new(kd, sig, bump_map))
    }

    pub fn plastic(kd: ColorTextureReference,
                   ks: ColorTextureReference,
                   rough: ScalarTextureReference,
                   bm: Option<ScalarTextureReference>) -> Material {
        Material::Plastic(PlasticMaterial::new(kd, ks, rough, bm))
    }

    pub fn measured(filename: String, b: Option<ScalarTextureReference>) -> Material {
        Material::Measured(MeasuredMaterial::new(filename, b))
    }

    pub fn mixed(m1: Arc<Material>, m2: Arc<Material>,
                 sc: ColorTextureReference) -> Material {
        Material::Mixed(MixMaterial::new(m1, m2, sc))
    }

    pub fn subsurface(scale: f32, k_r: ColorTextureReference,
                      sigma_a: ColorTextureReference,
                      sigma_prime_s: ColorTextureReference,
                      eta: ScalarTextureReference,
                      bm: Option<ScalarTextureReference>) -> Material {
        Material::Subsurface(
            SubsurfaceMaterial::new(scale, k_r, sigma_a, sigma_prime_s, eta, bm))
    }

    // !FIXME!
    pub fn broken() -> Material { Material::Broken }

    pub fn get_bsdf(&self, dg: DifferentialGeometry,
                    dgs: DifferentialGeometry) -> Option<BSDF> {
        match self {
            &Material::Matte(ref mat) => mat.get_bsdf(dg, dgs),
            &Material::Plastic(ref mat) => mat.get_bsdf(dg, dgs),
            &Material::Measured(ref mat) => mat.get_bsdf(dg, dgs),
            &Material::Mixed(ref mat) => mat.get_bsdf(dg, dgs),
            &Material::Subsurface(ref mat) => mat.get_bsdf(dg, dgs),
            _ => unimplemented!()
        }
    }

    pub fn get_bssrdf(&self, dg: DifferentialGeometry,
                      dgs: DifferentialGeometry) -> Option<BSSRDF> {
        match self {
            &Material::Subsurface(ref mat) => Some(mat.get_bssrdf(dg, dgs)),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_can_generate_normals_from_bump_maps() {
        // We need to try with some dummy bump maps here
        // once we get textures implemented
        unimplemented!()
    }
}
