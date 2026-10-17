param()
$ErrorActionPreference='Stop'
$projectRoot=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$priorEncoded=$env:CARGO_ENCODED_RUSTFLAGS
$priorFlags=$env:RUSTFLAGS
$flags=@()
if($priorEncoded){$flags+=@($priorEncoded.Split([char]31))}
elseif($priorFlags){$flags+=@($priorFlags.Split(' ',[StringSplitOptions]::RemoveEmptyEntries))}
foreach($entry in @(@($env:USERPROFILE,'/build-user'),@($projectRoot,'/src/koyomado'))){
 foreach($prefix in @($entry[0],$entry[0].Replace('\','/'))|Select-Object -Unique){
  $flags+='--remap-path-prefix='+$prefix+'='+$entry[1]
 }
}
try {
 $env:CARGO_ENCODED_RUSTFLAGS=$flags -join [char]31
 Push-Location $projectRoot
 try {
  & npm run tauri -- build --no-bundle
  if($LASTEXITCODE -ne 0){throw 'Release build failed.'}
  $target=if($env:CARGO_TARGET_DIR){$env:CARGO_TARGET_DIR}else{Join-Path $projectRoot 'src-tauri/target'}
  & node (Join-Path $PSScriptRoot 'verify-release-paths.mjs') (Join-Path $target 'release/koyomado.exe')
  if($LASTEXITCODE -ne 0){throw 'Release build path verification failed.'}
 } finally {Pop-Location}
} finally {
 $env:CARGO_ENCODED_RUSTFLAGS=$priorEncoded
 $env:RUSTFLAGS=$priorFlags
}
