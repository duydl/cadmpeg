"""Convert and compare three CATIA visibility states without conversion time limits."""
from pathlib import Path
import json
import subprocess
import sys

root = Path(__file__).resolve().parent
upstream, fixed = map(lambda p: str(Path(p).resolve()), sys.argv[1:3])
results = {}
for state, stem, prefix, renderer in [
    ("left", "curved-left-visible", "curved", "render_curved.py"),
    ("two", "curved-two-visible", "two", "render_two.py"),
    ("combined", "curved-combined-visible", "combined", "render_combined.py"),
]:
    for version, binary in [("upstream", upstream), ("pr", fixed)]:
        with (root/f"{prefix}-{version}.log").open("w") as log:
            subprocess.run([binary, "convert", f"{stem}.CATPart", "-o", f"{prefix}-{version}.step", "--force", "--report", f"{prefix}-{version}.report.json"], cwd=root, stdout=log, stderr=subprocess.STDOUT, check=True)
    with (root/f"{state}-geometry-diff.txt").open("w") as log:
        comparison = subprocess.run([fixed, "diff", f"{stem}.stp", f"{prefix}-pr.step", "--geometry"], cwd=root, stdout=log, stderr=subprocess.STDOUT)
    results[state] = {"geometry_agrees": comparison.returncode == 0}
    with (root/f"{state}-render.log").open("w") as log:
        subprocess.run([sys.executable, renderer], cwd=root, stdout=log, stderr=subprocess.STDOUT, check=True)
(root/"validation-summary.json").write_text(json.dumps(results, indent=2)+"\n")
print(json.dumps(results, indent=2))
assert all(v["geometry_agrees"] for v in results.values())
