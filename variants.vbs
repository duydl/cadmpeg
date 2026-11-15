Option Explicit
Dim app,doc,part,shapes,sel,fs,folder,i,state,stem
Set app=GetObject(,"CATIA.Application")
app.DisplayFileAlerts=False
Set fs=CreateObject("Scripting.FileSystemObject")
folder=fs.GetParentFolderName(WScript.ScriptFullName)
Set doc=app.ActiveDocument
Set part=doc.Part
Set shapes=part.HybridBodies.Item(1).HybridShapes
Set sel=doc.Selection
For state=0 To 1
 For i=1 To 3
  sel.Clear
  sel.Add shapes.Item(i)
  If (state=0 And i<3) Or (state=1 And i=3) Then
   sel.VisProperties.SetShow 0
  Else
   sel.VisProperties.SetShow 1
  End If
  sel.Clear
 Next
 If state=0 Then
  shapes.Item(1).Name="Left surface - visible"
  shapes.Item(2).Name="Right surface - visible"
  shapes.Item(3).Name="Combined surface - hidden"
  stem="curved-two-visible"
 Else
  shapes.Item(1).Name="Left surface - hidden"
  shapes.Item(2).Name="Right surface - hidden"
  shapes.Item(3).Name="Combined surface - visible"
  stem="curved-combined-visible"
 End If
 part.InWorkObject=shapes.Item(1)
 part.Update
 doc.SaveAs folder & "\" & stem & ".CATPart"
 doc.ExportData folder & "\" & stem & ".stp","stp"
Next
app.ActiveWindow.ActiveViewer.Reframe
