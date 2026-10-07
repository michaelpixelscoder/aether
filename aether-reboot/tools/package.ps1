param([switch]$SkipBuild)
$ErrorActionPreference='Stop'
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    if(!$SkipBuild){
        & cargo build --release --locked -p aether_game
        if($LASTEXITCODE -ne 0){throw 'Build Windows échoué'}
        & ./tools/bin/bevy.exe build --yes --release --locked -p aether_game web --bundle --wasm-opt=false
        if($LASTEXITCODE -ne 0){throw 'Build web échoué'}
    }
    & node tools/package.mjs
    if($LASTEXITCODE -ne 0){throw 'Packaging échoué'}
    $release=Get-Content dist/latest.json -Raw | ConvertFrom-Json
    foreach($target in @('windows','web')){
        $archive=Join-Path (Get-Location) "dist/aether-isles-$target-$($release.id).zip"
        if(!(Test-Path -LiteralPath $archive)){
            Compress-Archive -LiteralPath (Join-Path (Get-Location) "dist/$target/$($release.id)") -DestinationPath $archive -CompressionLevel Optimal
        }
        Get-FileHash -LiteralPath $archive -Algorithm SHA256
    }
}finally{Pop-Location}
