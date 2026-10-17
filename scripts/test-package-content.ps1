param()
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$fixtureRoot = Join-Path $projectRoot ('output/package-content-tests/' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path (Join-Path $fixtureRoot 'scripts'),(Join-Path $fixtureRoot 'docs'),(Join-Path $fixtureRoot 'src/assets/fonts'),(Join-Path $fixtureRoot 'third_party/opengameart-cc0-notification-sounds') -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'package-portable.ps1') -Destination (Join-Path $fixtureRoot 'scripts/package-portable.ps1')
'{"version":"1.1.0"}' | Set-Content -LiteralPath (Join-Path $fixtureRoot 'package.json') -Encoding UTF8
foreach ($file in @('LICENSE.txt','NOTICE','CHANGELOG.md','CHANGELOG.en.md','THIRD_PARTY_NOTICES.md','PRIVACY.md','PRIVACY.en.md','CODE_SIGNING_POLICY.md','README.en.md')) {
    ('Synthetic packaging fixture: ' + $file) | Set-Content -LiteralPath (Join-Path $fixtureRoot $file) -Encoding UTF8
}
foreach ($manual in @('Koyomado操作説明書.pdf','Koyomado-Manual.en.pdf')) {
    Copy-Item -LiteralPath (Join-Path $projectRoot 'docs/Koyomado操作説明書.pdf') -Destination (Join-Path $fixtureRoot "docs/$manual")
}
'Synthetic font license' | Set-Content -LiteralPath (Join-Path $fixtureRoot 'src/assets/fonts/OFL.txt')
'Synthetic sound notice' | Set-Content -LiteralPath (Join-Path $fixtureRoot 'third_party/opengameart-cc0-notification-sounds/NOTICE.txt')
foreach ($directory in @('docs','third_party/dependency-licenses','third_party/mpl-source','third_party/rust-backup')) {
    New-Item -ItemType Directory -Path (Join-Path $fixtureRoot $directory) -Force | Out-Null
}
foreach ($file in @('ASSET_PROVENANCE.md','docs/asset-manifest.json','third_party/dependency-licenses/index.json','third_party/mpl-source/index.json','third_party/rust-backup/zip-9.0.4-LICENSE.txt')) {
    'Synthetic provenance fixture' | Set-Content -LiteralPath (Join-Path $fixtureRoot $file)
}
Copy-Item -LiteralPath (Join-Path $fixtureRoot 'docs/Koyomado-Manual.en.pdf') -Destination (Join-Path $fixtureRoot 'docs/Koyomado.en.pdf')
'Not an executable. Synthetic packaging payload only.' | Set-Content -LiteralPath (Join-Path $fixtureRoot 'synthetic.exe')
& (Join-Path $fixtureRoot 'scripts/package-portable.ps1') -SkipBuild -ExecutablePath (Join-Path $fixtureRoot 'synthetic.exe')
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [IO.Compression.ZipFile]::OpenRead((Join-Path $fixtureRoot 'release/koyomado-v1.1.0-windows-portable.zip'))
try {
    foreach ($required in @('Koyomado-Manual.en.pdf','README.en.md','CHANGELOG.en.md','PRIVACY.en.md','ASSET_PROVENANCE.md','docs/asset-manifest.json','third_party/dependency-licenses/index.json','third_party/mpl-source/index.json','README.en.txt')) {
        if (-not $archive.GetEntry($required)) { throw "English release content missing from portable ZIP: $required" }
    }
    $entry = $archive.GetEntry('Koyomado-Manual.en.pdf')
    $stream = $entry.Open()
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $actual = ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-','') } finally {$stream.Dispose();$sha.Dispose()}
    if ($actual -ne (Get-FileHash -LiteralPath (Join-Path $fixtureRoot 'docs/Koyomado-Manual.en.pdf')).Hash) {throw 'English manual was modified while packaging'}
    Write-Output 'PASS: portable ZIP contains unchanged English manual, README, changelog and privacy policy.'
} finally {$archive.Dispose()}
