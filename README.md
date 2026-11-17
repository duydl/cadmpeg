# Public NIST FTC-07 combined recovery

Source fixture: corpus/nist-pmi/catia/nist_ftc_07_asme1_rd_ct5290_ra.CATPart from the cadmpeg NIST corpus. The publisher disclaimer is preserved in NIST-NOTICE.md. NIST authorship, license and source archive are documented at https://github.com/cadmpeg/cadmpeg/blob/main/corpus/nist-pmi/README.md. The reference STEP here was exported by CATIA from this public CATPart. All fixture results in this evidence set use public files.

revision.json records the exact source revisions and file hashes. Upstream and combined exports use cadmpeg convert INPUT --to step:ap242-e3 -o OUTPUT. Build each revision with cargo build -p cadmpeg --no-default-features --features catia,step. The combined source is at duydl/cadmpeg branch integration/ftc07-public-recovery.

The strict comparison uses a 0.001 mm threshold. Vertex and boundary incidence locate unique reference candidates. Each physical edge is sampled at fifteen interior points in both directions, with degenerate pole edges excluded. Surface interiors are projected in both directions and oriented normals compared. Surface samples use a three-by-three interior grid, increased for thin faces. The before and after JSON files retain every match and failure. Strict geometry and normal verification increases from 180/304 to 282/304; 22 reference faces remain outside the strict comparison. The reference declares 0.005 mm uncertainty.

OpenCascade accepts the combined export as one closed solid with 304 faces plus two native sheet faces. Its area is 391371.41838 mm² and volume 1726249.91267 mm³. The CATIA reference has area 391371.40782 mm² and volume 1726248.37899 mm³. opencascade.json records the measurements and public calosoe control.

Images are offscreen VTK renders of OpenCascade STEP imports. The three panels use the same orthographic camera and zoom. Meshing uses 0.1 mm linear and 0.5 rad angular deflection. The captions distinguish the two additional sheet faces. Re-render the supplied meshes with Python package vtk and python render.py . --offscreen.

## Audit of the 22 strict mismatches

The three-way rerun compares the combined decoder export, the fresh CATIA STEP export, and NIST's published STEP. The NIST STEP's inch coordinates are converted to millimetres for vertex matching. OpenCascade applies the declared units for curve and surface distance checks.

| Same-partition comparison at 0.001 mm | Unique topology candidates | Edges, surfaces and oriented normals verified |
| --- | ---: | ---: |
| Fresh CATIA STEP against original NIST STEP | 234 / 306 | 212 / 306 |
| Cadmpeg STEP against original NIST STEP | 236 / 306 | 206 / 306 |
| Cadmpeg STEP against fresh CATIA STEP | 304 / 304 | 282 / 304 |

Face and edge partition differences limit these face-by-face comparisons. The audit therefore compares every disputed edge directly with its original STEP counterpart and archived measurements from the public native CATPart.

Of the 22 faces, 16 involve circle differences alone, four involve spline differences alone, and two involve both. The 22 circular edge occurrences are exact circles in cadmpeg and polynomial approximations in both STEP exports. The two STEP polynomials agree to below 1.3e-9 mm in sampled checks. The decoder circles agree with the independently measured native circle and span to within a 1.162e-5 mm bound.

The six spline occurrences belong to three native splines. Cadmpeg agrees with the original STEP to below 1.1e-9 mm in sampled checks; the fresh CATIA export differs by about 0.003 mm. Archived native samples lie on the decoded spline bases within 9.1e-10 mm. Their contiguous active intervals reproduce the measured native lengths within 3.6e-9 mm. These numerical agreements describe the stored geometry and sampled computations, rather than physical part accuracy.

All 304 carrier surfaces and oriented normals pass the independent sampled surface check at 0.001 mm. The strict boundary-inclusive count remains 282/304. The current audit identifies STEP representation differences for the remaining 22, with native corroboration for each disputed edge. No production decoder change follows from these measurements. A new native-spline regression passes with the full 2,707-test CATIA suite at e815396ace50b7dd8b7aa3fd7941a62bd4cfc180.

The original NIST STEP and cadmpeg also retain two planar sheet faces with matching boundaries, surfaces and normals; the fresh CATIA export omits them. This accounts for the 306 versus 304 total shell-face counts.

mismatch-summary.json lists every affected face and the measured bounds. mismatch-edges.json retains each disputed edge, its endpoints, original STEP distances, and carrier residuals. The archived native measurement table and native circle/spline audit JSON provide the independent witnesses. The source fixture remains the public NIST CATPart recorded in revision.json.
