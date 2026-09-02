param(
    [Parameter(Mandatory = $true)]
    [string]$ArchivePath,

    [Parameter(Mandatory = $true)]
    [string]$ManualPath,

    [Parameter(Mandatory = $true)]
    [string]$CertificatePath,

    [Parameter(Mandatory = $true)]
    [string]$HashPath,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedVersion,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedThumbprint,

    [string]$ExpectedSubject = "CN=Y-TEC"
)

$ErrorActionPreference = "Stop"
$resolvedArchive = (Resolve-Path -LiteralPath $ArchivePath).Path
$resolvedManual = (Resolve-Path -LiteralPath $ManualPath).Path
$resolvedCertificate = (Resolve-Path -LiteralPath $CertificatePath).Path
$resolvedHash = (Resolve-Path -LiteralPath $HashPath).Path
$thumbprint = ($ExpectedThumbprint -replace "\s", "").ToUpperInvariant()
$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("koyomado-direct-verify-" + [guid]::NewGuid().ToString("N"))

try {
    $certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new($resolvedCertificate)
    if ($certificate.HasPrivateKey) {
        throw "公開用証明書に秘密鍵が含まれています。"
    }
    if ($certificate.Thumbprint -ne $thumbprint) {
        throw "公開用証明書のthumbprintが想定値と一致しません。"
    }
    if ($certificate.Subject -ne $ExpectedSubject) {
        throw "公開用証明書のSubjectが想定値と一致しません。"
    }

    New-Item -ItemType Directory -Force -Path $temporaryRoot | Out-Null
    Expand-Archive -LiteralPath $resolvedArchive -DestinationPath $temporaryRoot -Force

    $requiredFiles = @(
        "koyomado.exe",
        "Koyomado操作説明書.pdf",
        "LICENSE.txt",
        "NOTICE",
        "CHANGELOG.md",
        "THIRD_PARTY_NOTICES.md",
        "PRIVACY.md",
        "CODE_SIGNING_POLICY.md",
        "LINE_Seed_JP_OFL.txt",
        "NOTIFICATION_SOUNDS_CC0.txt",
        "はじめに.txt",
        "data\ここにデータが保存されます.txt"
    )
    foreach ($requiredFile in $requiredFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $temporaryRoot $requiredFile) -PathType Leaf)) {
            throw "直接配布ZIPに必要なファイルがありません: $requiredFile"
        }
    }

    $prohibitedFiles = Get-ChildItem -LiteralPath $temporaryRoot -Recurse -File |
        Where-Object {
            $_.Extension -in @(".pfx", ".p12", ".pvk", ".snk") -or
            $_.Name -in @("calendar-data.json", "window-state.json")
        }
    if ($prohibitedFiles) {
        throw "直接配布ZIPに秘密鍵または利用者データを含めてはいけません。"
    }

    $introduction = Get-Content -LiteralPath (Join-Path $temporaryRoot "はじめに.txt") -Raw
    if ($introduction -notmatch [regex]::Escape("Koyomado $ExpectedVersion")) {
        throw "直接配布ZIP内の版数案内が想定値と一致しません。"
    }

    $embeddedManualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $temporaryRoot "Koyomado操作説明書.pdf")).Hash
    $publishedManualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $resolvedManual).Hash
    if ($embeddedManualHash -ne $publishedManualHash) {
        throw "ZIP同梱の操作説明書と公開用PDFが一致しません。"
    }

    & (Join-Path $PSScriptRoot "verify-windows-signature.ps1") `
        -Path (Join-Path $temporaryRoot "koyomado.exe") `
        -ExpectedThumbprint $thumbprint `
        -ExpectedSubject $ExpectedSubject `
        -AllowUntrustedSelfSigned `
        -RequireTimestamp

    $expectedPublishedFiles = @($resolvedArchive, $resolvedManual, $resolvedCertificate)
    $hashEntries = @{}
    foreach ($line in Get-Content -LiteralPath $resolvedHash) {
        if ($line -notmatch '^(?<hash>[0-9a-fA-F]{64})  (?<name>[^\\/]+)$') {
            throw "SHA256SUMS.txtの形式が正しくありません。"
        }
        if ($hashEntries.ContainsKey($Matches.name)) {
            throw "SHA256SUMS.txtに同じファイル名が重複しています。"
        }
        $hashEntries[$Matches.name] = $Matches.hash.ToLowerInvariant()
    }
    if ($hashEntries.Count -ne $expectedPublishedFiles.Count) {
        throw "SHA256SUMS.txtの対象数が想定値と一致しません。"
    }
    foreach ($publishedFile in $expectedPublishedFiles) {
        $name = Split-Path -Leaf $publishedFile
        $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $publishedFile).Hash.ToLowerInvariant()
        if (-not $hashEntries.ContainsKey($name) -or $hashEntries[$name] -ne $actualHash) {
            throw "SHA-256が一致しません: $name"
        }
    }

    Write-Output "自己署名直接配布物を確認しました: $resolvedArchive"
    Write-Output "署名者: $($certificate.Subject)"
    Write-Output "SHA-256対象数: $($hashEntries.Count)"
}
finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        $resolvedTempBase = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath()).TrimEnd('\') + '\'
        $resolvedTemporary = [System.IO.Path]::GetFullPath($temporaryRoot)
        if (-not $resolvedTemporary.StartsWith($resolvedTempBase, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "検証用一時フォルダーがTEMP外を指しているため削除を中止しました: $resolvedTemporary"
        }
        Remove-Item -LiteralPath $resolvedTemporary -Recurse -Force
    }
}
