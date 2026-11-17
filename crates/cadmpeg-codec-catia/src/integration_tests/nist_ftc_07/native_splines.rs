// SPDX-License-Identifier: Apache-2.0

use super::*;
use cadmpeg_ir::eval::{decode::nurbs_curve_point_at, nurbs_curve_parameter_near_point};

#[test]
fn native_split_splines_preserve_catia_samples_and_lengths() {
    const POINT_TOLERANCE_MM: f64 = 1.0e-8;
    const LENGTH_TOLERANCE_MM: f64 = 1.0e-7;
    let witnesses = [
        (
            [544, 672],
            2.366_363_012_217_38,
            [
                [-139.565_874_328_891, -104.14, 71.347_630_628_916_1],
                [
                    -139.561_823_277_923,
                    -104.256_042_271_038,
                    70.771_719_768_072_5,
                ],
                [-139.513_599_811_522, -105.637_402_532_59, 69.823_862_741_57],
            ],
        ),
        (
            [551, 675],
            2.366_366_487_577_16,
            [
                [
                    -95.223_862_741_557_8,
                    -105.637_402_533_29,
                    95.063_599_811_494_4,
                ],
                [
                    -95.348_251_038_019,
                    -105.061_910_305_265,
                    95.083_690_320_201_2,
                ],
                [-96.747_630_629_742_9, -104.14, 95.115_874_329_181_9],
            ],
        ),
        (
            [554, 809],
            2.366_363_921_329_96,
            [
                [139.565_874_328_891, -104.14, -71.347_630_628_916_1],
                [
                    139.561_883_139_201,
                    -104.254_327_545_897,
                    -70.770_057_504_261_2,
                ],
                [
                    139.513_599_811_867,
                    -105.637_402_533_29,
                    -69.823_862_741_557_8,
                ],
            ],
        ),
    ];
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/nist-pmi/catia/nist_ftc_07_asme1_rd_ct5290_ra.CATPart");
    let decoded = CatiaCodec
        .decode(
            &mut Cursor::new(std::fs::read(path).expect("FTC-07")),
            &DecodeOptions::default(),
        )
        .expect("decode FTC-07");
    let geometry = &decoded.ir().geometry;
    for (ordinals, native_length, samples) in witnesses {
        let edges = ordinals.map(|ordinal| {
            geometry
                .edges
                .iter()
                .find(|edge| edge.id.as_str() == format!("catia:standard:edge#{ordinal}"))
                .expect("native spline occurrence")
        });
        let curves = edges.map(|edge| {
            let curve = geometry
                .curves
                .iter()
                .find(|curve| Some(&curve.id) == edge.curve())
                .expect("native spline carrier");
            let Some(SolvedCurveGeometry::Nurbs(curve)) = curve.geometry.solved() else {
                panic!("native spline");
            };
            curve
        });
        assert_eq!(curves[0], curves[1]);
        let mut intervals = edges.map(|edge| edge.param_range().expect("active interval").get());
        intervals.sort_by(|a, b| a[0].total_cmp(&b[0]));
        assert!((intervals[0][1] - intervals[1][0]).abs() < 1.0e-10);
        crate::test_support::with_service_context(|ctx| {
            let curve = curves[0];
            for [x, y, z] in samples {
                let witness = nurbs_curve_parameter_near_point(
                    ctx,
                    curve,
                    Point3::new(x, y, z),
                    POINT_TOLERANCE_MM,
                    (intervals[0][0] + intervals[1][1]) * 0.5,
                )
                .expect("native sample projection");
                assert!(witness.is_some());
            }
            let mut length = 0.0;
            for [start, end] in intervals {
                let mut previous = nurbs_curve_point_at(ctx, curve, start)
                    .expect("span start")
                    .get();
                for index in 1..=4096 {
                    let parameter = start + (end - start) * f64::from(index) / 4096.0;
                    let point = nurbs_curve_point_at(ctx, curve, parameter)
                        .expect("span sample")
                        .get();
                    length += previous.distance(point);
                    previous = point;
                }
            }
            assert!((length - native_length).abs() < LENGTH_TOLERANCE_MM);
        });
    }
}
