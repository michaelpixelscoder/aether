param([switch]$Web,[switch]$Release,[switch]$Browser)
$ErrorActionPreference='Stop'
$workspace=Split-Path -Parent $PSScriptRoot
Push-Location $workspace
try {
    New-Item -ItemType Directory -Force docs/evidence | Out-Null
    function Invoke-Check([string]$Name,[string]$Program,[string[]]$ArgumentList) {
        & $Program @ArgumentList 2>&1 | Tee-Object "docs/evidence/$Name.log"
        if ($LASTEXITCODE -ne 0) {throw "$Name a échoué : $LASTEXITCODE"}
    }
    Invoke-Check 'fmt' 'cargo' @('fmt','--all','--check')
    Invoke-Check 'clippy' 'cargo' @('clippy','--workspace','--all-targets','--locked','--','-D','warnings')
    Invoke-Check 'tests' 'cargo' @('test','--workspace','--locked')
    & cargo run -p aether_core --example world_catalog --locked --quiet | Set-Content -Encoding utf8NoBOM docs/evidence/world-catalog.json
    if($LASTEXITCODE -ne 0){throw 'Export du catalogue du monde échoué'}
    $buildArgs=@('build','--workspace','--locked')
    if($Release){$buildArgs+='--release'}
    Invoke-Check 'native-build' 'cargo' $buildArgs
    if($Web){
        Invoke-Check 'wasm-clippy' 'cargo' @('clippy','-p','aether_game','--target','wasm32-unknown-unknown','--locked','--','-D','warnings')
        $webArgs=@('build','--yes','--locked','-p','aether_game')
        if($Release){$webArgs+='--release'}
        $webArgs+=@('web','--bundle','--wasm-opt=false')
        Invoke-Check 'web-build' './tools/bin/bevy.exe' $webArgs
    }
    if($Browser){
        Push-Location tools
        try{& npx playwright test;if($LASTEXITCODE -ne 0){throw 'Tests navigateur échoués'}}finally{Pop-Location}
    }
    Write-Host 'Validation terminée. Preuves dans docs/evidence.'
} finally {Pop-Location}
