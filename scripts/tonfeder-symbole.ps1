# Zeichnet das Tonfeder-Zeichen (Text-Cursor aus Pegelbalken) als PNG:
# App-Symbol in 1024 px und die Tray-Bilder in 64 px fuer alle Zustaende.
# Aufruf aus dem Repo-Wurzelordner:  powershell -File scripts/tonfeder-symbole.ps1
# Danach:  bun tauri icon src-tauri/icons/tonfeder-1024.png
# Nur ASCII-Zeichen, weil Windows PowerShell 5.1 Dateien ohne BOM als ANSI liest.
Add-Type -AssemblyName System.Drawing

$repo = Split-Path $PSScriptRoot -Parent
$res = Join-Path $repo 'src-tauri\resources'
$ico = Join-Path $repo 'src-tauri\icons'

function Farbe([string]$hex) { [System.Drawing.ColorTranslator]::FromHtml($hex) }

# Zeichen auf einem 64er-Raster: Grundflaeche mit Ecken 3, zwei Querbalken, fuenf Pegelbalken
function Zeichne-Zeichen([int]$groesse, [string]$grund, [string]$tinte, [string]$rand, [string]$datei) {
  $bmp = New-Object System.Drawing.Bitmap $groesse, $groesse
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.Clear([System.Drawing.Color]::Transparent)
  $s = $groesse / 64.0

  $pfad = New-Object System.Drawing.Drawing2D.GraphicsPath
  $r = 3 * 2 * $s
  $pfad.AddArc(0, 0, $r, $r, 180, 90)
  $pfad.AddArc($groesse - $r, 0, $r, $r, 270, 90)
  $pfad.AddArc($groesse - $r, $groesse - $r, $r, $r, 0, 90)
  $pfad.AddArc(0, $groesse - $r, $r, $r, 90, 90)
  $pfad.CloseFigure()
  if ($grund) { $g.FillPath((New-Object System.Drawing.SolidBrush (Farbe $grund)), $pfad) }
  if ($rand) {
    $stift = New-Object System.Drawing.Pen (Farbe $rand), ([Math]::Max(1.5, 4 * $s))
    $innen = New-Object System.Drawing.Drawing2D.GraphicsPath
    $innen.AddRectangle((New-Object System.Drawing.RectangleF (2 * $s), (2 * $s), (60 * $s), (60 * $s)))
    $g.DrawPath($stift, $innen)
  }

  $pinsel = New-Object System.Drawing.SolidBrush (Farbe $tinte)
  $balken = @(
    @(18, 10, 28, 5), @(18, 49, 28, 5),
    @(18.2, 27, 3.6, 10), @(24.2, 22, 3.6, 20), @(30.2, 17, 3.6, 30), @(36.2, 21, 3.6, 22), @(42.2, 26, 3.6, 12)
  )
  foreach ($b in $balken) { $g.FillRectangle($pinsel, [float]($b[0] * $s), [float]($b[1] * $s), [float]($b[2] * $s), [float]($b[3] * $s)) }

  $g.Dispose()
  $bmp.Save($datei, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  "geschrieben: $datei"
}

$lime = '#a3e635'; $tinte = '#0a0f0b'; $blau = '#7dd3fc'; $gelb = '#fcd34d'
$gedaempft = '#a8bcab'; $dunkelgrau = '#4b5a50'; $hell = '#f4f6f3'

# App-Symbol (Quelle fuer "bun tauri icon")
Zeichne-Zeichen 1024 $lime $tinte $null (Join-Path $ico 'tonfeder-1024.png')

# Tray, dunkle Taskleiste (helle Symbole)
Zeichne-Zeichen 64 $gedaempft $tinte $null (Join-Path $res 'tray_idle.png')
Zeichne-Zeichen 64 $lime $tinte $null (Join-Path $res 'tray_recording.png')
Zeichne-Zeichen 64 $blau $tinte $null (Join-Path $res 'tray_transcribing.png')
Zeichne-Zeichen 64 $gelb $tinte $null (Join-Path $res 'tray_idle_warning.png')

# Tray, helle Taskleiste (dunkle Symbole)
Zeichne-Zeichen 64 $dunkelgrau $hell $null (Join-Path $res 'tray_idle_dark.png')
Zeichne-Zeichen 64 $lime $tinte $null (Join-Path $res 'tray_recording_dark.png')
Zeichne-Zeichen 64 $blau $tinte $null (Join-Path $res 'tray_transcribing_dark.png')
Zeichne-Zeichen 64 $gelb $tinte $null (Join-Path $res 'tray_idle_warning_dark.png')

# Farbige Variante (Linux) und allgemeine Bilder
Zeichne-Zeichen 64 $lime $tinte $null (Join-Path $res 'handy.png')
Zeichne-Zeichen 64 $lime $tinte $null (Join-Path $res 'recording.png')
Zeichne-Zeichen 64 $blau $tinte $null (Join-Path $res 'transcribing.png')
Zeichne-Zeichen 64 $gelb $tinte $null (Join-Path $res 'handy_warning.png')
