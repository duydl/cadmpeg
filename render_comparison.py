# SPDX-License-Identifier: Apache-2.0
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

assert len(re.findall(rb"ADVANCED_FACE\(", (root/"before.step").read_bytes())) == 0
assert len(re.findall(rb"ADVANCED_FACE\(", (root/"after.step").read_bytes())) == 1
torus = gp_Torus(gp_Ax3(gp_Pnt(1, 2, 3), gp_Dir(0, 0, 1)), 7, 2)
expected = BRepBuilderAPI_MakeFace(torus, 0.25, 1.0, -0.6, -0.2).Face()
reader = STEPControl_Reader()
assert int(reader.ReadFile(str(root/"after.step"))) == 1
assert reader.TransferRoots()
recovered = reader.OneShape()
assert len(list(shapes(recovered, TopAbs_FACE))) == 1
expected_actors, recovered_actors = actors(expected), actors(recovered)
camera = vtk.vtkCamera()
bounds = expected_actors[0].GetBounds()
center = [(bounds[i] + bounds[i+1])/2 for i in (0, 2, 4)]
span = math.dist(bounds[::2], bounds[1::2])
camera.SetFocalPoint(*center)
camera.SetPosition(center[0]+1.8*span*math.cos(0.625), center[1]+1.8*span*math.sin(0.625), center[2]-span)
camera.SetViewUp(0, 0, 1)
camera.ParallelProjectionOn()
camera.SetParallelScale(0.64*span)
camera.SetClippingRange(0.01, 8*span)
window = vtk.vtkRenderWindow()
window.SetOffScreenRendering(1)
window.SetSize(1680, 620)
window.SetMultiSamples(8)
panels = [
    ("SYNTHETIC EXPECTATION", "Analytic torus patch", expected_actors),
    ("UPSTREAM", "0 bounded faces", []),
    ("CHART DOMAIN FIX", "1 bounded face / 4 edges", recovered_actors),
]
for i, (label, detail, items) in enumerate(panels):
    renderer = vtk.vtkRenderer()
    renderer.SetViewport(i/3, 0, (i+1)/3, 1)
    renderer.SetBackground(0.96, 0.97, 0.98)
    renderer.SetActiveCamera(camera)
    for actor in items:
        renderer.AddActor(actor)
    text(renderer, label, 0.06, 0.91, 23, (0.08, 0.16, 0.21))
    text(renderer, detail, 0.06, 0.075, 19, (0.20, 0.30, 0.36))
    if not items:
        text(renderer, "Trim rejected at active ranges", 0.08, 0.47, 20, (0.55, 0.25, 0.18))
    window.AddRenderer(renderer)
window.Render()
capture = vtk.vtkWindowToImageFilter()
capture.SetInput(window)
capture.Update()
writer = vtk.vtkPNGWriter()
writer.SetFileName(str(root/"comparison.png"))
writer.SetInputConnection(capture.GetOutputPort())
writer.Write()
print(root/"comparison.png")
