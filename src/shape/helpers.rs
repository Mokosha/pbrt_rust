use diff_geom::DifferentialGeometry;
use shape::ShapeBase;
use geometry::point::Point;
use geometry::vector::Dot;
use geometry::vector::Vector;
use geometry::normal::Normal;
use geometry::normal::Normalize;
use transform::transform::ApplyTransform;

// Note: This is the part where the math kind of escapes me as I haven't
// actually taken a course on differential geometry. For that, the book recommends
// the book by A. Gray:
// Modern differential geometry of curves and surfaces
// ISBN: 0849378729

pub fn compute_dg(shape: &ShapeBase, u: f32, v: f32, p_hit: Point,
                  dpdu: Vector, dpdv: Vector,
                  d2pduu: Vector, d2pduv: Vector, d2pdvv: Vector)
                  -> DifferentialGeometry {
    // Compute coefficients for final forms
    let _ee = dpdu.dot(&dpdu);
    let _ff = dpdu.dot(&dpdv);
    let _gg = dpdv.dot(&dpdv);
    let _nn : Vector = dpdu.clone().cross(&dpdv).normalize();
    let _e = _nn.dot(&d2pduu);
    let _f = _nn.dot(&d2pduv);
    let _g = _nn.dot(&d2pdvv);

    // Compute dn/du and dn/dv from fundamental form coefficients
    let inveeggff2 = 1.0 / (_ee * _gg - _ff * _ff);
    let dndu = Normal::from((_f*_ff - _e*_gg) * inveeggff2 * &dpdu +
                            (_e*_ff - _f*_ee) * inveeggff2 * &dpdv);
    let dndv = Normal::from((_g*_ff - _f*_gg) * inveeggff2 * &dpdu +
                            (_f*_ff - _g*_ee) * inveeggff2 * &dpdv);

    // Initialize DifferentialGeometry from parametric information
    let o2w = &(shape.object2world);

    DifferentialGeometry::new_with(
        o2w.xf(p_hit), o2w.xf(dpdu), o2w.xf(dpdv), o2w.xf(dndu),
        o2w.xf(dndv), u, v, Some(shape.clone()))
}
