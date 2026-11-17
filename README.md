# Public NIST FTC-07 combined recovery

Source fixture: corpus/nist-pmi/catia/nist_ftc_07_asme1_rd_ct5290_ra.CATPart from the cadmpeg NIST corpus. The publisher disclaimer is preserved in NIST-NOTICE.md. NIST authorship, license and source archive are documented at https://github.com/cadmpeg/cadmpeg/blob/main/corpus/nist-pmi/README.md. The reference STEP here was exported by CATIA from this public CATPart. All fixture results in this evidence set use public files.

revision.json records the exact source revisions and file hashes. Upstream and combined exports use cadmpeg convert INPUT --to step:ap242-e3 -o OUTPUT. Build each revision with cargo build -p cadmpeg --no-default-features --features catia,step. The combined source is at duydl/cadmpeg branch integration/ftc07-public-recovery.

The strict comparison uses a 0.001 mm threshold. Vertex and boundary incidence locate unique reference candidates. Each physical edge is sampled at fifteen interior points in both directions, with degenerate pole edges excluded. Surface interiors are projected in both directions and oriented normals compared. Surface samples use a three-by-three interior grid, increased for thin faces. The before and after JSON files retain every match and failure. Strict geometry and normal verification increases from 180/304 to 282/304; 22 reference faces remain outside the strict comparison. The reference declares 0.005 mm uncertainty.

OpenCascade accepts the combined export as one closed solid with 304 faces plus two native sheet faces. Its area is 391371.41838 mm² and volume 1726249.91267 mm³. The CATIA reference has area 391371.40782 mm² and volume 1726248.37899 mm³. opencascade.json records the measurements and public calosoe control.

Images are offscreen VTK renders of OpenCascade STEP imports. The three panels use the same orthographic camera and zoom. Meshing uses 0.1 mm linear and 0.5 rad angular deflection. The captions distinguish the two additional sheet faces. Re-render the supplied meshes with Python package vtk and python render.py . --offscreen.
