param([switch]$Browser)
$ErrorActionPreference='Stop'
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    & rustup show active-toolchain
    if($LASTEXITCODE -ne 0){throw 'Installer Rust via rustup, puis les outils C++ MSVC et le SDK Windows.'}
    & rustup component add rustfmt clippy
    if($LASTEXITCODE -ne 0){throw 'Installation rustfmt/Clippy échouée'}
    & rustup target add wasm32-unknown-unknown
    if($LASTEXITCODE -ne 0){throw 'Installation WASM échouée'}
    $cli='tools/bin/bevy.exe'
    $expected='218533EA83CAF09FFC696EF860512BD90B3FE16F46B71CA4476C085AC9F8FEB0'
    if(!(Test-Path -LiteralPath $cli)){
        New-Item -ItemType Directory -Force tools/bin | Out-Null
        Invoke-WebRequest -Uri 'https://github.com/TheBevyFlock/bevy_cli/releases/download/cli-v0.1.0-alpha.2/bevy-x86_64-pc-windows-msvc-v0.1.0-alpha.2.exe' -OutFile $cli
    }
    if((Get-FileHash -LiteralPath $cli -Algorithm SHA256).Hash -ne $expected){throw 'Empreinte Bevy CLI inattendue'}
    & cargo fetch --locked
    if($LASTEXITCODE -ne 0){throw 'Téléchargement des dépendances échoué'}
    if($Browser){
        Push-Location tools
        try {
            & npm ci --ignore-scripts
            if($LASTEXITCODE -ne 0){throw 'Installation Playwright échouée'}
            & npx playwright install chromium
            if($LASTEXITCODE -ne 0){throw 'Installation Chromium échouée'}
        }finally{Pop-Location}
    }
    Write-Host 'Outils prêts. Aucun outil auteur ou serveur distant nécessaire.'
}finally{Pop-Location}
