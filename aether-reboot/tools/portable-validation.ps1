$ErrorActionPreference='Stop'
$workspace=Split-Path -Parent $PSScriptRoot
$release=Get-Content -LiteralPath (Join-Path $workspace 'dist/latest.json') -Raw | ConvertFrom-Json
$directory=Join-Path ([IO.Path]::GetTempPath()) ('Aether portable '+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $directory | Out-Null
$archive=Join-Path $workspace "dist/aether-isles-windows-$($release.id).zip"
Expand-Archive -LiteralPath $archive -DestinationPath $directory
$package=Join-Path $directory $release.id
# An unrelated working directory must not override the package's own assets.
New-Item -ItemType Directory -Path (Join-Path $directory 'assets/textures') -Force | Out-Null
Set-Content -LiteralPath (Join-Path $directory 'assets/textures/cedar.png') -Value 'Not an image: deliberate working-directory decoy.'
$manifest=Get-Content -LiteralPath (Join-Path $package 'MANIFEST.json') -Raw | ConvertFrom-Json
foreach($entry in $manifest.files){
    $file=Join-Path $package $entry.path
    if((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256){throw "Checksum invalide : $($entry.path)"}
}
$previousData=$env:AETHER_DATA
$report=Join-Path $workspace 'docs/evidence/portable-qa.json'
try {
    $env:AETHER_DATA=Join-Path $directory 'user data'
    $process=Start-Process -FilePath (Join-Path $package 'Aether-Isles.exe') -ArgumentList @('--qa','--qa-output',('"'+$report+'"')) -WorkingDirectory $directory -WindowStyle Hidden -PassThru
    $timer=[Diagnostics.Stopwatch]::StartNew()
    while(!$process.HasExited){if($timer.Elapsed.TotalSeconds -gt 90){$process.Kill();throw 'Timeout du package portable'};Start-Sleep -Milliseconds 250}
    $process.WaitForExit()
    $data=Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    if($process.ExitCode -ne 0 -or !$data.qa_finished -or $data.qa_failure){throw "Échec portable : $($data.qa_failure)"}
    @{id=$release.id;extracted_directory=$package;working_directory=$directory;working_directory_decoy_ignored=$true;verified_files=$manifest.files.Count;exit_code=$process.ExitCode;elapsed_seconds=$timer.Elapsed.TotalSeconds;save_exists=(Test-Path -LiteralPath (Join-Path $env:AETHER_DATA 'session.json'));git_used=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $workspace 'docs/evidence/portable-validation.json')
    Write-Host 'Archive extraite hors workspace : checksums, assets, parcours et sauvegarde validés.'
}finally{$env:AETHER_DATA=$previousData}
