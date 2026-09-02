param(
    [Parameter(Mandatory = $true)]
    [string]$CertificateThumbprint,
    [string]$TimestampUrl = "http://timestamp.digicert.com",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$releaseRoot = Join-Path $projectRoot "release"
$packageJson = Get-Content -LiteralPath (Join-Path $projectRoot "package.json") -Raw | ConvertFrom-Json
$version = [string]$packageJson.version
$executablePath = Join-Path $projectRoot "src-tauri\target\release\koyomado.exe"
$archivePath = Join-Path $releaseRoot "koyomado-v$version-windows-portable.zip"
$certificatePath = Join-Path $releaseRoot "Y-TEC-CodeSigning-Public.cer"
$manualPath = Join-Path $releaseRoot "Koyomado.pdf"
$hashPath = Join-Path $releaseRoot "SHA256SUMS.txt"
$thumbprint = ($CertificateThumbprint -replace "\s", "").ToUpperInvariant()
$signingStagingPath = Join-Path $releaseRoot ".staging-signed-direct-$PID"
$signedExecutablePath = Join-Path $signingStagingPath "koyomado.exe"

Push-Location $projectRoot
try {
    if (-not $SkipBuild) {
        npm run tauri:build
        if ($LASTEXITCODE -ne 0) { throw "Tauriビルドに失敗しました。" }
    }
    if (-not (Test-Path -LiteralPath $executablePath -PathType Leaf)) {
        throw "ビルド済み実行ファイルが見つかりません: $executablePath"
    }

    New-Item -ItemType Directory -Force -Path $signingStagingPath | Out-Null
    Copy-Item -LiteralPath $executablePath -Destination $signedExecutablePath

    & (Join-Path $PSScriptRoot "sign-windows-artifact.ps1") `
        -Path $signedExecutablePath `
        -CertificateThumbprint $thumbprint `
        -ExpectedSubject "CN=Y-TEC" `
        -RequireSelfSignedNonExportable `
        -TimestampUrl $TimestampUrl
    & (Join-Path $PSScriptRoot "verify-windows-signature.ps1") `
        -Path $signedExecutablePath `
        -ExpectedThumbprint $thumbprint `
        -ExpectedSubject "CN=Y-TEC" `
        -AllowUntrustedSelfSigned `
        -RequireTimestamp

    & (Join-Path $PSScriptRoot "package-portable.ps1") `
        -SkipBuild `
        -ExecutablePath $signedExecutablePath
    Copy-Item -LiteralPath (Join-Path $projectRoot "docs\Koyomado操作説明書.pdf") -Destination $manualPath -Force

    $certificate = Get-ChildItem -LiteralPath "Cert:\CurrentUser\My" |
        Where-Object { $_.Thumbprint -eq $thumbprint } |
        Select-Object -First 1
    if (-not $certificate) { throw "公開鍵を出力する証明書が見つかりません。" }
    Export-Certificate -Cert $certificate -FilePath $certificatePath -Type CERT -Force | Out-Null

    $publishedFiles = @($archivePath, $manualPath, $certificatePath)
    $hashLines = foreach ($publishedFile in $publishedFiles) {
        $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $publishedFile).Hash.ToLowerInvariant()
        "$hash  $(Split-Path -Leaf $publishedFile)"
    }
    [System.IO.File]::WriteAllLines(
        $hashPath,
        [string[]]$hashLines,
        [System.Text.UTF8Encoding]::new($false)
    )

    & (Join-Path $PSScriptRoot "verify-direct-release.ps1") `
        -ArchivePath $archivePath `
        -ManualPath $manualPath `
        -CertificatePath $certificatePath `
        -HashPath $hashPath `
        -ExpectedVersion $version `
        -ExpectedThumbprint $thumbprint `
        -ExpectedSubject "CN=Y-TEC"

    Write-Output "自己署名直接配布物を作成しました:"
    $publishedFiles + $hashPath | ForEach-Object { Write-Output $_ }
}
finally {
    Pop-Location
    if (Test-Path -LiteralPath $signingStagingPath) {
        $resolvedRelease = [System.IO.Path]::GetFullPath($releaseRoot).TrimEnd('\') + '\'
        $resolvedStaging = [System.IO.Path]::GetFullPath($signingStagingPath)
        if (-not $resolvedStaging.StartsWith($resolvedRelease, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "署名用一時フォルダーがrelease外を指しているため削除を中止しました: $resolvedStaging"
        }
        Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
    }
}
