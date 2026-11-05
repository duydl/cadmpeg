// SPDX-License-Identifier: Apache-2.0
//! Bounded polynomial approximation of a native rolling-ball face carrier.

use cadmpeg_ir::document::CadIr;
use cadmpeg_ir::eval::{self, admission::EvaluationAdmission};
use cadmpeg_ir::geometry::nurbs::{NurbsPoleGrid, NurbsSurface, NurbsSurfaceAxis};
use cadmpeg_ir::geometry::SurfaceGeometry;
use cadmpeg_ir::math::Point3;

use crate::wire::records::{ConsolidatedFamily, ConsolidatedRecord};

const U_DEGREE: usize = 3;
const V_DEGREE: usize = 5;
const HOLDOUT_STEPS: usize = 20;
const MAX_FIT_ERROR_MM: f64 = 0.001;
const MIN_PIVOT: f64 = 1.0e-12;
const LOOP_CLOSURE_MM: f64 = 0.001;
const MAX_PCURVE_CANDIDATES: usize = 256;
const MAX_CHART_ENDPOINT_SLACK: f64 = 0.00001;

fn bernstein(degree: usize, t: f64) -> Vec<f64> {
    let mut row = Vec::with_capacity(degree + 1);
    let mut coefficient = 1.0;
    for index in 0..=degree {
        row.push(coefficient * t.powi(index as i32) * (1.0 - t).powi((degree - index) as i32));
        if index < degree {
            coefficient *= (degree - index) as f64 / (index + 1) as f64;
        }
    }
    row
}

fn solve(matrix: &[Vec<f64>], values: &[f64]) -> Option<Vec<f64>> {
    let count = values.len();
    if matrix.len() != count || matrix.iter().any(|row| row.len() != count) {
        return None;
    }
    let mut rows = matrix
        .iter()
        .zip(values)
        .map(|(row, value)| {
            let mut augmented = row.clone();
            augmented.push(*value);
            augmented
        })
        .collect::<Vec<_>>();
    for column in 0..count {
        let pivot = (column..count).max_by(|&left, &right| {
            rows[left][column]
                .abs()
                .total_cmp(&rows[right][column].abs())
        })?;
        if rows[pivot][column].abs() < MIN_PIVOT {
            return None;
        }
        rows.swap(column, pivot);
        let scale = rows[column][column];
        for value in &mut rows[column] {
            *value /= scale;
        }
        for other in 0..count {
            if other == column {
                continue;
            }
            let factor = rows[other][column];
            for entry in column..=count {
                rows[other][entry] -= factor * rows[column][entry];
            }
        }
    }
    Some(rows.into_iter().map(|row| row[count]).collect())
}

pub(super) fn fit(
    ir: &CadIr,
    carrier: &SurfaceGeometry,
    u_range: [f64; 2],
    v_range: [f64; 2],
) -> Option<NurbsSurface> {
    if !u_range
        .iter()
        .chain(v_range.iter())
        .all(|value| value.is_finite())
        || u_range[0] >= u_range[1]
        || v_range[0] >= v_range[1]
    {
        return None;
    }
    let evaluate = |u: f64, v: f64| {
        let point = eval::model_surface_point(EvaluationAdmission::Standard, ir, carrier, u, v)
            .ok()?
            .get();
        Some([point.x, point.y, point.z])
    };
    let u_basis = (0..=U_DEGREE)
        .map(|index| bernstein(U_DEGREE, index as f64 / U_DEGREE as f64))
        .collect::<Vec<_>>();
    let v_basis = (0..=V_DEGREE)
        .map(|index| bernstein(V_DEGREE, index as f64 / V_DEGREE as f64))
        .collect::<Vec<_>>();
    let mut samples = [[[0.0; 3]; V_DEGREE + 1]; U_DEGREE + 1];
    for (i, row) in samples.iter_mut().enumerate() {
        let u = u_range[0] + (u_range[1] - u_range[0]) * i as f64 / U_DEGREE as f64;
        for (j, point) in row.iter_mut().enumerate() {
            let v = v_range[0] + (v_range[1] - v_range[0]) * j as f64 / V_DEGREE as f64;
            *point = evaluate(u, v)?;
        }
    }
    let mut along_u = [[[0.0; 3]; V_DEGREE + 1]; U_DEGREE + 1];
    for j in 0..=V_DEGREE {
        for axis in 0..3 {
            let values = samples.iter().map(|row| row[j][axis]).collect::<Vec<_>>();
            for (i, value) in solve(&u_basis, &values)?.into_iter().enumerate() {
                along_u[i][j][axis] = value;
            }
        }
    }
    let mut poles = [[[0.0; 3]; V_DEGREE + 1]; U_DEGREE + 1];
    for i in 0..=U_DEGREE {
        for axis in 0..3 {
            let values = along_u[i]
                .iter()
                .map(|point| point[axis])
                .collect::<Vec<_>>();
            for (j, value) in solve(&v_basis, &values)?.into_iter().enumerate() {
                poles[i][j][axis] = value;
            }
        }
    }
    for i in 0..=HOLDOUT_STEPS {
        let tu = i as f64 / HOLDOUT_STEPS as f64;
        let u = u_range[0] + (u_range[1] - u_range[0]) * tu;
        let bu = bernstein(U_DEGREE, tu);
        for j in 0..=HOLDOUT_STEPS {
            let tv = j as f64 / HOLDOUT_STEPS as f64;
            let v = v_range[0] + (v_range[1] - v_range[0]) * tv;
            let bv = bernstein(V_DEGREE, tv);
            let actual = evaluate(u, v)?;
            let mut predicted = [0.0; 3];
            for (row, weight_u) in poles.iter().zip(&bu) {
                for (point, weight_v) in row.iter().zip(&bv) {
                    for axis in 0..3 {
                        predicted[axis] += weight_u * weight_v * point[axis];
                    }
                }
            }
            let squared = (0..3)
                .map(|axis| (actual[axis] - predicted[axis]).powi(2))
                .sum::<f64>();
            if !squared.is_finite() || squared.sqrt() > MAX_FIT_ERROR_MM {
                return None;
            }
        }
    }
    let points = poles
        .iter()
        .map(|row| {
            row.iter()
                .map(|point| Point3::new(point[0], point[1], point[2]))
                .collect()
        })
        .collect();
    NurbsSurface::new(
        NurbsSurfaceAxis::new(
            U_DEGREE as u32,
            vec![
                u_range[0], u_range[0], u_range[0], u_range[0], u_range[1], u_range[1], u_range[1],
                u_range[1],
            ],
            false,
        ),
        NurbsSurfaceAxis::new(
            V_DEGREE as u32,
            vec![
                v_range[0], v_range[0], v_range[0], v_range[0], v_range[0], v_range[0], v_range[1],
                v_range[1], v_range[1], v_range[1], v_range[1], v_range[1],
            ],
            false,
        ),
        NurbsPoleGrid::Polynomial { rows: points },
        false,
    )
    .ok()
}

pub(super) fn closed_patch_domain(
    ir: &CadIr,
    carrier: &SurfaceGeometry,
    data: &[u8],
    records: &[ConsolidatedRecord],
    source_index: usize,
    refusal: &mut crate::nurbs::LaneRefusals,
) -> Option<([f64; 2], [f64; 2])> {
    let mut candidates = Vec::new();
    let source_records = records
        .iter()
        .filter(|record| record.source_index() == source_index)
        .cloned()
        .collect::<Vec<_>>();
    for family in [ConsolidatedFamily::A, ConsolidatedFamily::B] {
        for pcurve in
            crate::wire::records::family_pcurves_from_records(data, &source_records, family)
        {
            let candidate = (|| {
                let (knots, points, first, second) = pcurve.native_lanes();
                let knots = knots
                    .into_iter()
                    .map(|value| value.get())
                    .collect::<Vec<_>>();
                let points = points
                    .into_iter()
                    .map(|value| value.get())
                    .collect::<Vec<_>>();
                let first = first
                    .into_iter()
                    .map(|value| value.get())
                    .collect::<Vec<_>>();
                let second = second
                    .into_iter()
                    .map(|value| value.get())
                    .collect::<Vec<_>>();
                let geometry = crate::assemble::quintic_jet_pcurve(
                    crate::wire::records::ConsolidatedPcurve::DEGREE,
                    &knots,
                    &points,
                    (&first, &second),
                    refusal,
                    pcurve.pos,
                )?;
                let endpoints = pcurve.range.endpoints();
                let mut lifted = Vec::new();
                for parameter in endpoints {
                    let uv = eval::decode::pcurve_uv(
                        EvaluationAdmission::Standard,
                        &geometry,
                        parameter,
                    )
                    .ok()?
                    .get();
                    if uv.v < -MAX_CHART_ENDPOINT_SLACK || uv.v > 1.0 + MAX_CHART_ENDPOINT_SLACK {
                        return None;
                    }
                    let chart_v = uv.v.clamp(0.0, 1.0);
                    let point = eval::model_surface_point(
                        EvaluationAdmission::Standard,
                        ir,
                        carrier,
                        uv.u,
                        chart_v,
                    )
                    .ok()?
                    .get();
                    lifted.push(([uv.u, chart_v], [point.x, point.y, point.z]));
                }
                Some(lifted)
            })();
            if let Some(lifted) = candidate {
                candidates.push(lifted);
                if candidates.len() > MAX_PCURVE_CANDIDATES {
                    return None;
                }
            }
        }
    }
    let mut closed = Vec::new();
    for (index, first) in candidates.iter().enumerate() {
        for second in candidates.iter().skip(index + 1) {
            let distance = |left: &[f64; 3], right: &[f64; 3]| {
                (0..3)
                    .map(|axis| (left[axis] - right[axis]).powi(2))
                    .sum::<f64>()
                    .sqrt()
            };
            let same = distance(&first[0].1, &second[0].1).max(distance(&first[1].1, &second[1].1));
            let opposite =
                distance(&first[0].1, &second[1].1).max(distance(&first[1].1, &second[0].1));
            if same.min(opposite) < LOOP_CLOSURE_MM {
                let u = first
                    .iter()
                    .chain(second)
                    .map(|row| row.0[0])
                    .collect::<Vec<_>>();
                let v = first
                    .iter()
                    .chain(second)
                    .map(|row| row.0[1])
                    .collect::<Vec<_>>();
                closed.push((
                    [
                        u.iter().copied().fold(f64::INFINITY, f64::min),
                        u.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    ],
                    [
                        v.iter().copied().fold(f64::INFINITY, f64::min),
                        v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    ],
                ));
            }
        }
    }
    (closed.len() == 1).then(|| closed[0])
}

#[cfg(test)]
mod tests {
    use super::fit;
    use cadmpeg_ir::document::CadIr;
    use cadmpeg_ir::geometry::nurbs::{NurbsPoleGrid, NurbsSurface, NurbsSurfaceAxis};
    use cadmpeg_ir::geometry::{SolvedSurfaceGeometry, SurfaceGeometry};
    use cadmpeg_ir::math::Point3;

    fn surface(u_degree: u32, u_coordinates: &[f64]) -> SurfaceGeometry {
        let controls = u_coordinates
            .iter()
            .map(|coordinate| {
                vec![
                    Point3::new(*coordinate, 0.0, 0.0),
                    Point3::new(*coordinate, 1.0, 0.0),
                ]
            })
            .collect::<Vec<_>>();
        let nurbs = NurbsSurface::new(
            NurbsSurfaceAxis::new(
                u_degree,
                if u_degree == 3 {
                    vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
                } else {
                    vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0]
                },
                false,
            ),
            NurbsSurfaceAxis::new(1, vec![0.0, 0.0, 1.0, 1.0], false),
            NurbsPoleGrid::Polynomial { rows: controls },
            false,
        )
        .expect("synthetic surface");
        SurfaceGeometry::Solved(SolvedSurfaceGeometry::Nurbs(nurbs))
    }

    #[test]
    fn bounded_patch_fit_accepts_cubic_and_rejects_quartic() {
        let ir = CadIr::empty();
        let cubic = surface(3, &[0.0, 0.0, 0.0, 1.0]);
        let quartic = surface(4, &[0.0, 0.0, 0.0, 0.0, 1.0]);
        assert!(fit(&ir, &cubic, [0.0, 1.0], [0.0, 1.0]).is_some());
        assert!(fit(&ir, &quartic, [0.0, 1.0], [0.0, 1.0]).is_none());
    }
}
