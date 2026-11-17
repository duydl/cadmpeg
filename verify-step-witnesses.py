from pathlib import Path
import argparse,json,re,hashlib,subprocess
from OCP.STEPControl import STEPControl_Reader
from OCP.StepGeom import StepGeom_BSplineCurveWithKnots
from OCP.StepToGeom import StepToGeom
from OCP.GeomAPI import GeomAPI_ProjectPointOnCurve
from OCP.gp import gp_Pnt
parser=argparse.ArgumentParser(description="Verify upstream witnesses against the public NIST STEP, independently of the CATIA decoder.")
parser.add_argument("repository",type=Path)
parser.add_argument("--output",type=Path,default=Path("step-witness-provenance.json"))
args=parser.parse_args()
root=args.repository.resolve()
fixture=root/'corpus/nist-pmi/step/nist_ftc_07_asme1_ap242-e2.stp'
text=subprocess.check_output(['git','show','9c4e34b5d:crates/cadmpeg-codec-catia/src/integration_tests/nist_ftc_07.rs'],cwd=root,text=True)
reader=STEPControl_Reader();assert int(reader.ReadFile(str(fixture)))==1
model=reader.StepModel();curves=[]
labels=[int(x) for x in re.findall(r"^#(\d+)\s*=",fixture.read_text(),re.M)]
assert len(labels)==model.NbEntities()
records=dict(re.findall(r"^#(\d+)\s*=\s*(.*?);",fixture.read_text(),re.M|re.S))
for i in range(1,model.NbEntities()+1):
 entity=model.Value(i)
 if isinstance(entity,StepGeom_BSplineCurveWithKnots):
  label=labels[i-1]
  assert records[str(label)].startswith("B_SPLINE_CURVE_WITH_KNOTS")
  curves.append((label,StepToGeom.MakeBSplineCurve_s(entity)))
rows=[]
for ordinal in [418,425,436,443,700,728,753,761]:
 values=[float(x.strip().replace('_','')) for x in re.search(rf'\({ordinal}, \[(.*?)\]\)',text)[1].split(',')]
 point=gp_Pnt(*(x/25.4 for x in values));best=None
 for label,curve in curves:
  projection=GeomAPI_ProjectPointOnCurve(point,curve,curve.FirstParameter(),curve.LastParameter())
  if not projection.NbPoints():continue
  item=(projection.LowerDistance()*25.4,label,projection.LowerDistanceParameter())
  if best is None or item<best:best=item
 assert best is not None and best[0]<1e-8,(ordinal,best)
 rows.append(dict(edge=ordinal,point_mm=values,step_curve_entity=best[1],parameter=best[2],gap_mm=best[0]))
result=dict(fixture='corpus/nist-pmi/step/'+fixture.name,sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(),units='INCH, 25.4 mm per unit',source_constants_revision='9c4e34b5d25e2822b8120c54e680523779d451ea',method='OpenCascade StepToGeom reads the public STEP spline entities directly; GeomAPI projects each original upstream witness in source inches.',rows=rows)
args.output.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
