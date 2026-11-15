"""Render the synthetic torus case with OpenCascade and VTK."""
from pathlib import Path
import math
import re
import vtk
from OCP.BRep import BRep_Tool
from OCP.BRepAdaptor import BRepAdaptor_Curve
from OCP.BRepBuilderAPI import BRepBuilderAPI_MakeFace
from OCP.BRepMesh import BRepMesh_IncrementalMesh
from OCP.gp import gp_Ax3, gp_Dir, gp_Pnt, gp_Torus
from OCP.STEPControl import STEPControl_Reader
from OCP.TopAbs import TopAbs_FACE, TopAbs_EDGE
from OCP.TopExp import TopExp_Explorer
from OCP.TopLoc import TopLoc_Location
from OCP.TopoDS import TopoDS

root = Path(__file__).parent

def shapes(shape, kind):
    it = TopExp_Explorer(shape, kind)
    while it.More():
        yield it.Current()
        it.Next()

def poly(points, cells, color, surface=True):
    xyz, indices = vtk.vtkPoints(), vtk.vtkCellArray()
    for point in points:
        xyz.InsertNextPoint(*point)
    for cell in cells:
        indices.InsertNextCell(len(cell))
        for index in cell:
            indices.InsertCellPoint(index)
    data = vtk.vtkPolyData()
    data.SetPoints(xyz)
    if surface:
        data.SetPolys(indices)
    else:
        data.SetLines(indices)
    mapper = vtk.vtkPolyDataMapper()
    mapper.SetInputData(data)
    actor = vtk.vtkActor()
    actor.SetMapper(mapper)
    actor.GetProperty().SetColor(*color)
    actor.GetProperty().SetAmbient(0.35)
    actor.GetProperty().SetDiffuse(0.65)
    actor.GetProperty().SetLineWidth(2.5)
    return actor

def actors(shape):
    BRepMesh_IncrementalMesh(shape, 0.003, False, 0.08, True)
    result = []
    for face in shapes(shape, TopAbs_FACE):
        loc = TopLoc_Location()
        mesh = BRep_Tool.Triangulation_s(TopoDS.Face(face), loc)
        assert mesh is not None
        points = [mesh.Node(i).Transformed(loc.Transformation()).Coord()
                  for i in range(1, mesh.NbNodes()+1)]
        triangles = [tuple(j-1 for j in mesh.Triangle(i).Get())
                     for i in range(1, mesh.NbTriangles()+1)]
        result.append(poly(points, triangles, (0.23, 0.60, 0.78)))
    for edge in shapes(shape, TopAbs_EDGE):
        curve = BRepAdaptor_Curve(TopoDS.Edge(edge))
        lo, hi = curve.FirstParameter(), curve.LastParameter()
        points = [curve.Value(lo+(hi-lo)*i/128).Coord() for i in range(129)]
        result.append(poly(points, [list(range(129))], (0.08, 0.21, 0.28), False))
    return result

def text(renderer, value, x, y, size, color):
    actor = vtk.vtkTextActor()
    actor.SetInput(value)
    actor.GetPositionCoordinate().SetCoordinateSystemToNormalizedViewport()
    actor.SetPosition(x, y)
    actor.GetTextProperty().SetFontSize(size)
    actor.GetTextProperty().SetColor(*color)
    renderer.AddViewProp(actor)


import json
from OCP.BRepCheck import BRepCheck_Analyzer
from OCP.BRepGProp import BRepGProp
from OCP.GProp import GProp_GProps
from OCP.TopAbs import TopAbs_SHELL, TopAbs_SOLID
rows=[]
window=vtk.vtkRenderWindow()
window.SetOffScreenRendering(1)
window.SetSize(1680,620)
window.SetMultiSamples(8)
camera=vtk.vtkCamera()
camera.SetFocalPoint(20,10,10)
camera.SetPosition(75,-75,80)
camera.SetViewUp(0,0,1)
camera.ParallelProjectionOn()
camera.SetParallelScale(32)
camera.SetClippingRange(.01,400)
for i,(label,name) in enumerate([("UPSTREAM BASE", "two-upstream.step"),("PR #1272", "two-pr.step"),("CATIA REFERENCE", "curved-two-visible.stp")]):
    path=root/name
    r=STEPControl_Reader()
    assert int(r.ReadFile(str(path)))==1 and r.TransferRoots(), name
    shape=r.OneShape()
    faces=list(shapes(shape,TopAbs_FACE))
    props=GProp_GProps()
    BRepGProp.SurfaceProperties_s(shape,props)
    row=dict(file=name,faces=len(faces),shells=len(list(shapes(shape,TopAbs_SHELL))),solids=len(list(shapes(shape,TopAbs_SOLID))),valid=BRepCheck_Analyzer(shape).IsValid(),area=props.Mass())
    rows.append(row)
    renderer=vtk.vtkRenderer()
    renderer.SetViewport(i/3,0,(i+1)/3,1)
    renderer.SetBackground(.96,.97,.98)
    renderer.SetActiveCamera(camera)
    for actor in actors(shape):renderer.AddActor(actor)
    text(renderer,label,.06,.91,23,(.08,.16,.21))
    text(renderer,f"{len(faces)} face(s) / {row['shells']} shell(s)",.06,.11,18,(.20,.30,.36))
    text(renderer,f"Area {props.Mass():.3f} mm²",.06,.06,17,(.20,.30,.36))
    window.AddRenderer(renderer)
window.Render()
capture=vtk.vtkWindowToImageFilter()
capture.SetInput(window)
capture.Update()
writer=vtk.vtkPNGWriter()
writer.SetFileName(str(root/'two-visible-body-comparison.png'))
writer.SetInputConnection(capture.GetOutputPort())
writer.Write()
(root/'two-opencascade-results.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows,indent=2))
