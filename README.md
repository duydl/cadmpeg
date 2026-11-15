# CATIA visible-body selection evidence

Synthetic fixtures and captures for cadmpeg/cadmpeg#1272.

The part stores a left surface, a right surface, and a combined two-face surface. Three saved variants select the left surface, the two separate surfaces, or the combined surface. The in-work object also changes independently of visibility.

| Visible result | Upstream OCCT faces / shells | PR OCCT faces / shells | CATIA reference faces / shells |
| --- | --- | --- | --- |
| Left surface | 8 / 3 | 1 / 1 | 1 / 1 |
| Two separate surfaces | 8 / 3 | 2 / 2 | 2 / 2 |
| Combined surface | 8 / 3 | 2 / 1 | 2 / 1 |

OpenCascade accepts all nine exports as valid. Face counts include standalone surface carriers. Each PR export agrees with its CATIA reference under `cadmpeg diff --geometry`; the reports are included.

## Reproduce

Start CATIA V5 in an interactive Windows desktop session. Run `cscript //nologo curved-specimen.vbs`, followed by `cscript //nologo variants.vbs`, in a writable directory. These scripts create spline profiles and lofts from explicit coordinates, copy the surfaces as results, set visibility, save CATParts, and export CATIA STEP references.

Build cadmpeg at the two commits in `manifest.json`, using `cargo build -p cadmpeg --no-default-features --features catia,step`. Keep the binaries separately. With Python packages `cadquery-ocp` and `vtk` available, run `python validate.py /path/to/upstream/cadmpeg /path/to/pr/cadmpeg` in this directory. Conversion commands have no time limit.

## Capture method

`catia-curved-feature-tree.png` is a CATIA V5 screenshot captured over RDP and cropped to the CATIA window. Cropping removes the desktop and taskbar; the window contents are unchanged. Its object names identify the initial visible and hidden states.

For each comparison image, OpenCascade imports the complete upstream STEP, PR STEP, and CATIA STEP reference. OpenCascade meshes the faces, and VTK renders them off-screen with the same orthographic camera and zoom. All imported faces are shown. The captions report OpenCascade face and shell counts and measured surface area. The images are model renders, rather than screenshots of an OpenCascade viewer application.

The three comparison images show body selection and grouping. IR support-role retention is covered by the CATIA regression test. Caching behavior is separate from these images.
