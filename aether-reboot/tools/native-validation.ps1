param([string]$Executable='target/release/aether_game.exe',[switch]$CloseTests)
$ErrorActionPreference='Stop'
$workspace=Split-Path -Parent $PSScriptRoot
$binary=(Resolve-Path -LiteralPath (Join-Path $workspace $Executable)).Path
$evidence=Join-Path $workspace 'docs/evidence'
$previousData=$env:AETHER_DATA
$results=@()
function Start-Candidate([string[]]$Arguments,[string]$Name){
    $env:AETHER_DATA=Join-Path $evidence "native-data-$Name"
    Start-Process -FilePath $binary -ArgumentList $Arguments -WorkingDirectory $workspace -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $evidence "$Name.stdout.log") -RedirectStandardError (Join-Path $evidence "$Name.stderr.log")
}
function Wait-Candidate($Process,[int]$Seconds=90){
    $timer=[Diagnostics.Stopwatch]::StartNew();$peak=0L;$private=0L
    while(!$Process.HasExited){
        $Process.Refresh();$peak=[Math]::Max($peak,$Process.WorkingSet64);$private=[Math]::Max($private,$Process.PrivateMemorySize64)
        if($timer.Elapsed.TotalSeconds -gt $Seconds){$Process.Kill();throw 'Le candidat a dépassé le timeout'}
        Start-Sleep -Milliseconds 250
    }
    $Process.WaitForExit()
    if($Process.ExitCode -ne 0){throw "Code de sortie $($Process.ExitCode)"}
    @{elapsed_seconds=$timer.Elapsed.TotalSeconds;sampled_peak_working_set_bytes=$peak;sampled_peak_private_bytes=$private;exit_code=$Process.ExitCode}
}
try {
    foreach($mode in @('qa','benchmark')){
        $report=Join-Path $evidence "native-$mode.json"
        $process=Start-Candidate @("--$mode",'--qa-output',('"'+$report+'"')) "native-$mode"
        $measure=Wait-Candidate $process
        $data=Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
        if(!$data.qa_finished -or $null -ne $data.qa_failure){throw "Échec $mode : $($data.qa_failure)"}
        $measure.mode=$mode;$measure.report="native-$mode.json";$results+=$measure
        $results | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'native-process-measurements.json')
    }
    if($CloseTests){
        $closeStarted=Get-Date
        $process=Start-Candidate @('--qa','--qa-output',('"'+(Join-Path $evidence 'native-close-interrupted.json')+'"')) 'close-success'
        # Wait for an actual running session, including a fresh autosave.
        # A fixed delay could close the loading screen on a cold asset cache.
        $sessionPath=Join-Path $env:AETHER_DATA 'session.json'
        $ready=$false
        $timer=[Diagnostics.Stopwatch]::StartNew()
        while(!$ready -and $timer.Elapsed.TotalSeconds -lt 45){
            $process.Refresh()
            if($process.HasExited){throw 'Le jeu a quitté avant le test de fermeture'}
            if(Test-Path -LiteralPath $sessionPath){
                $savedFile=Get-Item -LiteralPath $sessionPath
                if($savedFile.LastWriteTime -ge $closeStarted){
                    try {$saved=Get-Content -LiteralPath $sessionPath -Raw | ConvertFrom-Json;$ready=$saved.version -eq 2 -and $saved.vessels.Count -gt 0} catch {$ready=$false}
                }
            }
            if(!$ready){Start-Sleep -Milliseconds 250}
        }
        if(!$ready){throw 'Aucune sauvegarde récente avant fermeture'}
        if(!$process.CloseMainWindow()){throw 'Aucune fenêtre à fermer'}
        $measure=Wait-Candidate $process 20
        $session=Get-Content -LiteralPath (Join-Path $env:AETHER_DATA 'session.json') -Raw | ConvertFrom-Json
        if($session.version -ne 2 -or $session.vessels.Count -lt 1){throw 'Fermeture sans sauvegarde valide'}
        $measure.mode='close-success';$measure.saved_version=$session.version;$results+=$measure
    }
    $results | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $evidence 'native-process-measurements.json')
    $results | Format-Table
} finally {$env:AETHER_DATA=$previousData}
