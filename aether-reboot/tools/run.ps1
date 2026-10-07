param([switch]$Release,[switch]$Gallery,[switch]$QA,[switch]$Benchmark)
$ErrorActionPreference='Stop'
Push-Location (Split-Path -Parent $PSScriptRoot)
$oldData=$env:AETHER_DATA
try {
    $arguments=@('run','--locked')
    if($Gallery){$arguments+=@('-p','aether_view','--example','gallery')}else{$arguments+=@('-p','aether_game')}
    if($Release){$arguments+='--release'}
    $arguments+='--'
    if($QA -or $Benchmark){$env:AETHER_DATA=Join-Path (Get-Location) 'docs/evidence/qa-saves'}
    if($QA){$arguments+=@('--qa','--qa-output','docs/evidence/native-qa.json')}
    if($Benchmark){$arguments+=@('--benchmark','--qa-output','docs/evidence/native-benchmark.json')}
    & cargo @arguments
    exit $LASTEXITCODE
}finally{$env:AETHER_DATA=$oldData;Pop-Location}
