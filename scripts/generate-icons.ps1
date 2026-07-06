param(
    [string]$OutputDir = "assets/icons"
)

Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root $OutputDir
New-Item -ItemType Directory -Force -Path $out | Out-Null

$pngPath = Join-Path $out "oxidenms.png"
$icoPath = Join-Path $out "oxidenms.ico"

$size = 256
$bitmap = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$graphics.Clear([System.Drawing.Color]::Transparent)

function New-Path {
    param([array]$Points)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddPolygon($Points)
    return $path
}

function Pt([float]$x, [float]$y) {
    return [System.Drawing.PointF]::new($x, $y)
}

function Add-RoundedRect {
    param(
        [System.Drawing.Drawing2D.GraphicsPath]$Path,
        [System.Drawing.RectangleF]$Rect,
        [float]$Radius
    )

    $d = $Radius * 2
    $Path.AddArc($Rect.X, $Rect.Y, $d, $d, 180, 90)
    $Path.AddArc($Rect.Right - $d, $Rect.Y, $d, $d, 270, 90)
    $Path.AddArc($Rect.Right - $d, $Rect.Bottom - $d, $d, $d, 0, 90)
    $Path.AddArc($Rect.X, $Rect.Bottom - $d, $d, $d, 90, 90)
    $Path.CloseFigure()
}

$bg = New-Object System.Drawing.Drawing2D.GraphicsPath
Add-RoundedRect $bg ([System.Drawing.RectangleF]::new(0, 0, 256, 256)) 52
$graphics.FillPath([System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(255, 9, 17, 31)), $bg)

$shield = New-Path @(
    (Pt 128 26),
    (Pt 202 58),
    (Pt 202 121),
    (Pt 198 148),
    (Pt 187 174),
    (Pt 168 198),
    (Pt 128 231),
    (Pt 88 198),
    (Pt 69 174),
    (Pt 58 148),
    (Pt 54 121),
    (Pt 54 58)
)
$shieldBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    [System.Drawing.PointF]::new(48, 32),
    [System.Drawing.PointF]::new(208, 224),
    [System.Drawing.Color]::FromArgb(255, 27, 54, 93),
    [System.Drawing.Color]::FromArgb(255, 31, 122, 90)
)
$blend = New-Object System.Drawing.Drawing2D.ColorBlend
$blend.Positions = [float[]](0, 0.55, 1)
$blend.Colors = [System.Drawing.Color[]]@(
    [System.Drawing.Color]::FromArgb(255, 27, 54, 93),
    [System.Drawing.Color]::FromArgb(255, 11, 110, 138),
    [System.Drawing.Color]::FromArgb(255, 31, 122, 90)
)
$shieldBrush.InterpolationColors = $blend
$graphics.FillPath($shieldBrush, $shield)
$graphics.DrawPath([System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(255, 217, 247, 255), 6), $shield)

$linkPen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(255, 217, 247, 255), 13)
$linkPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$linkPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
$linkPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$graphics.DrawLine($linkPen, 88, 112, 168, 112)
$graphics.DrawLine($linkPen, 128, 82, 128, 174)
$graphics.DrawLine($linkPen, 94, 164, 128, 112)
$graphics.DrawLine($linkPen, 128, 112, 162, 164)

$nodeBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    [System.Drawing.PointF]::new(76, 70),
    [System.Drawing.PointF]::new(180, 184),
    [System.Drawing.Color]::FromArgb(255, 217, 247, 255),
    [System.Drawing.Color]::FromArgb(255, 125, 255, 178)
)
$outlinePen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(255, 9, 17, 31), 6)
foreach ($p in @((Pt 128 82), (Pt 88 112), (Pt 168 112), (Pt 94 164), (Pt 162 164))) {
    $rect = [System.Drawing.RectangleF]::new($p.X - 18, $p.Y - 18, 36, 36)
    $graphics.FillEllipse($nodeBrush, $rect)
    $graphics.DrawEllipse($outlinePen, $rect)
}

$basePen = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(191, 125, 255, 178), 8)
$basePen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$basePen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
$graphics.DrawLine($basePen, 64, 202, 192, 202)

$bitmap.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()

$pngBytes = [System.IO.File]::ReadAllBytes($pngPath)
$ico = New-Object System.Collections.Generic.List[byte]
$ico.AddRange([byte[]](0,0,1,0,1,0))
$ico.AddRange([byte[]](0,0,0,0,1,0,32,0))
$ico.AddRange([System.BitConverter]::GetBytes([uint32]$pngBytes.Length))
$ico.AddRange([System.BitConverter]::GetBytes([uint32]22))
$ico.AddRange($pngBytes)
[System.IO.File]::WriteAllBytes($icoPath, $ico.ToArray())

Write-Output "Generated $pngPath"
Write-Output "Generated $icoPath"
