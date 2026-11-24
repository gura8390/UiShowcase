param(
    [string]$OutputPath = (Join-Path $PSScriptRoot "app.ico")
)

Add-Type -AssemblyName System.Drawing

function New-IconBitmap {
    param([int]$Size)
    $bmp = New-Object System.Drawing.Bitmap $Size, $Size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)

    $pad = [math]::Max(1, [int][math]::Round($Size * 0.06))
    $rect = New-Object System.Drawing.RectangleF $pad, $pad, ($Size - 2 * $pad), ($Size - 2 * $pad)
    $radius = [float][math]::Max(2.0, $Size * 0.20)

    # Rounded square path
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $radius * 2
    $path.AddArc($rect.X, $rect.Y, $d, $d, 180, 90)
    $path.AddArc($rect.Right - $d, $rect.Y, $d, $d, 270, 90)
    $path.AddArc($rect.Right - $d, $rect.Bottom - $d, $d, $d, 0, 90)
    $path.AddArc($rect.X, $rect.Bottom - $d, $d, $d, 90, 90)
    $path.CloseFigure()

    # Clean Fluent blue background (#0F6CBD)
    $bgBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 15, 108, 189))
    $g.FillPath($bgBrush, $path)

    # Inner clean UI grid mark representing a structured desktop workspace
    $whiteBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $softBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(190, 255, 255, 255))

    $m = $Size * 0.22
    $gap = [math]::Max(1.0, $Size * 0.07)
    $w = $Size - 2 * $m
    $h = $Size - 2 * $m

    # Left nav column
    $navW = $w * 0.30
    $g.FillRectangle($softBrush, [float]$m, [float]$m, [float]$navW, [float]$h)

    # Right top header bar
    $rightX = $m + $navW + $gap
    $rightW = $w - $navW - $gap
    $topH = $h * 0.34
    $g.FillRectangle($whiteBrush, [float]$rightX, [float]$m, [float]$rightW, [float]$topH)

    # Right bottom workspace pane
    $botY = $m + $topH + $gap
    $botH = $h - $topH - $gap
    $g.FillRectangle($whiteBrush, [float]$rightX, [float]$botY, [float]$rightW, [float]$botH)

    $bgBrush.Dispose()
    $whiteBrush.Dispose()
    $softBrush.Dispose()
    $path.Dispose()
    $g.Dispose()
    return $bmp
}

$sizes = @(16, 24, 32, 48, 64, 256)
$pngList = @()

foreach ($s in $sizes) {
    $bmp = New-IconBitmap -Size $s
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $pngList += ,@($s, $ms.ToArray())
    $ms.Dispose()
    $bmp.Dispose()
}

$outDir = Split-Path -Parent $OutputPath
if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null
}

$fs = [System.IO.File]::Open($OutputPath, [System.IO.FileMode]::Create)
$bw = New-Object System.IO.BinaryWriter $fs

# ICONDIR header
$bw.Write([UInt16]0)
$bw.Write([UInt16]1)
$bw.Write([UInt16]$pngList.Count)

$offset = 6 + (16 * $pngList.Count)
foreach ($entry in $pngList) {
    $sz = $entry[0]
    $bytes = $entry[1]
    $bw.Write([Byte]($(if ($sz -ge 256) { 0 } else { $sz })))
    $bw.Write([Byte]($(if ($sz -ge 256) { 0 } else { $sz })))
    $bw.Write([Byte]0)
    $bw.Write([Byte]0)
    $bw.Write([UInt16]1)
    $bw.Write([UInt16]32)
    $bw.Write([UInt32]$bytes.Length)
    $bw.Write([UInt32]$offset)
    $offset += $bytes.Length
}

foreach ($entry in $pngList) {
    $bw.Write([byte[]]$entry[1])
}

$bw.Dispose()
$fs.Dispose()
Write-Host "Generated icon: $OutputPath"
