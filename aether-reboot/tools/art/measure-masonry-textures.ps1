$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$stats = @()
foreach ($file in @('dressed-limestone-r11.png', 'dressed-masonry-r42-albedo.png', 'dressed-masonry-r42-normal.png', 'dressed-masonry-r42-roughness.png')) {
    $bitmap = [System.Drawing.Bitmap]::new([string](Resolve-Path "assets/textures/$file"))
    $raw = @(0.0, 0.0, 0.0); $linear = @(0.0, 0.0, 0.0)
    $min = @(1.0, 1.0, 1.0); $max = @(0.0, 0.0, 0.0); $count = 0
    for ($y = 0; $y -lt $bitmap.Height; $y += 4) {
        for ($x = 0; $x -lt $bitmap.Width; $x += 4) {
            $p = $bitmap.GetPixel($x, $y)
            $v = @(($p.R / 255.0), ($p.G / 255.0), ($p.B / 255.0))
            for ($i = 0; $i -lt 3; $i++) {
                $raw[$i] += $v[$i]
                $linear[$i] += $(if ($v[$i] -le .04045) { $v[$i] / 12.92 } else { [Math]::Pow(($v[$i] + .055) / 1.055, 2.4) })
                $min[$i] = [Math]::Min($min[$i], $v[$i]); $max[$i] = [Math]::Max($max[$i], $v[$i])
            }
            $count++
        }
    }
    $stats += @{ file = $file; width = $bitmap.Width; height = $bitmap.Height; samples = $count; mean_raw = @($raw | ForEach-Object { $_ / $count }); mean_linear = @($linear | ForEach-Object { $_ / $count }); min = $min; max = $max }
    $bitmap.Dispose()
}
$stats | ConvertTo-Json -Depth 8 | Set-Content .dream-loop/masonry-texture-statistics.json
$stats | ConvertTo-Json -Depth 8
