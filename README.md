# Synthetic CATIA joined sheet

This part was authored in a blank CATIA V5-6R2022 document. Three synthetic spline profiles define two lofts. CATIA joins the lofts along their common profile and saves the resulting sheet in a new CATPart.

Use this same input with each cadmpeg build.

```sh
cadmpeg convert joined-sheet.CATPart -o candidate.step
```

Upstream `1372cebf` writes zero bounded faces. The recovery branch at `f486e678` writes two bounded faces. Both faces match the CATIA STEP at 0.001 mm with the same winding. The two faces share one edge with opposite traversal.

The reference STEP is CATIA's export with a neutral filename in the header. OpenCascade tessellated each exported surface and sampled the edge curves. VTK used a common orthographic camera. Orange marks the shared edge.
