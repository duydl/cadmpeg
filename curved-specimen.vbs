Option Explicit
Dim fs, path, log, catia, doc, part, geo, factory, i, j, x, y, z
Dim point, spline, sections(2), lofts(1), noClosingPoint, joined, sel, result, resultSet
Dim target, reference, alerts
Set fs = CreateObject("Scripting.FileSystemObject")
path = fs.GetParentFolderName(WScript.ScriptFullName)
Set log = fs.CreateTextFile(path & "\build-native.log", True)
Sub Check(label, code, message)
    log.WriteLine label & " " & code & " " & message
    If code <> 0 Then
        log.Close
        WScript.Quit 1
    End If
End Sub
On Error Resume Next
Set catia = GetObject(, "CATIA.Application")
Check "CATIA", Err.Number, Err.Description
alerts = catia.DisplayFileAlerts
catia.DisplayFileAlerts = False
Set doc = catia.Documents.Add("Part")
Check "NEW", Err.Number, Err.Description
Set part = doc.Part
Check "PART", Err.Number, Err.Description
Set geo = part.HybridBodies.Add()
Check "GEOMETRICAL SET", Err.Number, Err.Description
geo.Name = "Synthetic profiles"
Set factory = part.HybridShapeFactory
Check "FACTORY", Err.Number, Err.Description
Set noClosingPoint = Nothing
For i = 0 To 2
    x = 20 * i
    Set spline = factory.AddNewSpline()
    Check "NEW SPLINE", Err.Number, Err.Description
    For j = 0 To 4
        y = 5 * j
        z = 0.024 * y * y + 0.005 * x * y + 0.012 * x * x
        Set point = factory.AddNewPointCoord(x, y, z)
        Check "POINT", Err.Number, Err.Description
        geo.AppendHybridShape point
        Check "APPEND POINT", Err.Number, Err.Description
        spline.AddPoint part.CreateReferenceFromObject(point)
        Check "ADD POINT", Err.Number, Err.Description
    Next
    geo.AppendHybridShape spline
    Check "APPEND SPLINE", Err.Number, Err.Description
    part.UpdateObject spline
    Check "PROFILE " & i, Err.Number, Err.Description
    Set sections(i) = part.CreateReferenceFromObject(spline)
Next
For i = 0 To 1
    Set lofts(i) = factory.AddNewLoft()
    lofts(i).AddSectionToLoft sections(i), 1, noClosingPoint
    lofts(i).AddSectionToLoft sections(i+1), 1, noClosingPoint
    geo.AppendHybridShape lofts(i)
    part.UpdateObject lofts(i)
    Check "LOFT " & i, Err.Number, Err.Description
Next
Set joined = factory.AddNewJoin(part.CreateReferenceFromObject(lofts(0)), part.CreateReferenceFromObject(lofts(1)))
geo.AppendHybridShape joined
part.UpdateObject joined
Check "JOIN", Err.Number, Err.Description

Dim resultShapes(2), originals(2), idx
Set originals(0)=lofts(0)
Set originals(1)=lofts(1)
Set originals(2)=joined
Set result = catia.Documents.Add("Part")
Set resultSet=result.Part.HybridBodies.Add()
resultSet.Name="Visible result selection"
For idx=0 To 2
 Set sel=doc.Selection
 sel.Clear
 sel.Add originals(idx)
 sel.Copy
 Set sel=result.Selection
 sel.Clear
 sel.Add resultSet
 sel.PasteSpecial "CATPrtResultWithOutLink"
 result.Part.Update
 Check "PASTE " & idx, Err.Number, Err.Description
 Set resultShapes(idx)=resultSet.HybridShapes.Item(resultSet.HybridShapes.Count)
Next
resultShapes(0).Name="Left surface - visible"
resultShapes(1).Name="Right surface - hidden"
resultShapes(2).Name="Combined surface - hidden"
For idx=0 To 2
 sel.Clear
 sel.Add resultShapes(idx)
 If idx=0 Then
  sel.VisProperties.SetShow 0
 Else
  sel.VisProperties.SetShow 1
 End If
 sel.Clear
Next
result.Part.InWorkObject=resultShapes(2)
result.Part.Update
result.SaveAs path & "\curved-left-visible.CATPart"
Check "SAVE", Err.Number, Err.Description
result.ExportData path & "\curved-left-visible.stp","stp"
Check "EXPORT", Err.Number, Err.Description
catia.Visible=True
catia.Left=0
catia.Top=0
catia.Width=1600
catia.Height=960
catia.ActiveWindow.ActiveViewer.Reframe
catia.ActiveWindow.ActiveViewer.Update
catia.DisplayFileAlerts=alerts
log.WriteLine "DONE"
log.Close
