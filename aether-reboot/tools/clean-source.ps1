$ErrorActionPreference='Stop'
$workspace=Split-Path -Parent $PSScriptRoot
$copy=Join-Path ([IO.Path]::GetTempPath()) ('Aether source clean '+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $copy | Out-Null
foreach($item in @('Cargo.toml','Cargo.lock','rust-toolchain.toml','.cargo','crates','assets','web')){
    Copy-Item -LiteralPath (Join-Path $workspace $item) -Destination $copy -Recurse
}
$previousTarget=$env:CARGO_TARGET_DIR
$timer=[Diagnostics.Stopwatch]::StartNew()
try {
    $env:CARGO_TARGET_DIR=Join-Path $workspace 'target'
    Push-Location $copy
    try {& cargo check --workspace --all-targets --locked --offline 2>&1 | Tee-Object (Join-Path $workspace 'docs/evidence/clean-source.log');$result=$LASTEXITCODE}
    finally {Pop-Location}
    @{source_directory=$copy;command='cargo check --workspace --all-targets --locked --offline';dependency_cache_shared=$true;target_directory=$env:CARGO_TARGET_DIR;elapsed_seconds=$timer.Elapsed.TotalSeconds;exit_code=$result;git_used=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $workspace 'docs/evidence/clean-source.json')
    if($result -ne 0){throw 'La copie propre ne compile pas'}
} finally {$env:CARGO_TARGET_DIR=$previousTarget}
