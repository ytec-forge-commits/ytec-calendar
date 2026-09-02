param(
    [Parameter(Mandatory = $true)]
    [string]$Path,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedThumbprint,

    [string]$ExpectedSubject,

    [switch]$AllowUntrustedSelfSigned,

    [switch]$RequireTimestamp
)

$ErrorActionPreference = "Stop"
$resolvedPath = (Resolve-Path -LiteralPath $Path).Path
$thumbprint = ($ExpectedThumbprint -replace "\s", "").ToUpperInvariant()
$signature = Get-AuthenticodeSignature -LiteralPath $resolvedPath

if (-not $signature.SignerCertificate) {
    throw "Authenticode署名がありません: $resolvedPath"
}
if ($signature.SignerCertificate.Thumbprint -ne $thumbprint) {
    throw "署名者のthumbprintが想定値と一致しません。"
}
if ($ExpectedSubject -and $signature.SignerCertificate.Subject -ne $ExpectedSubject) {
    throw "署名者のSubjectが想定値と一致しません。"
}
if ($RequireTimestamp -and -not $signature.TimeStamperCertificate) {
    throw "RFC 3161タイムスタンプを確認できません。"
}

if ($signature.Status -ne "Valid") {
    $selfSigned = $signature.SignerCertificate.Subject -eq $signature.SignerCertificate.Issuer
    $allowedStatus = $signature.Status -in @("UnknownError", "NotTrusted")
    if (-not ($AllowUntrustedSelfSigned -and $selfSigned -and $allowedStatus)) {
        throw "署名検証に失敗しました: $($signature.Status)"
    }
    Write-Warning "自己署名の暗号署名と署名者は一致しましたが、このPCでは信頼されていません。公開時も利用者環境で警告が出る可能性があります。"
}

Write-Output "署名を確認しました: $resolvedPath"
Write-Output "状態: $($signature.Status)"
Write-Output "署名者: $($signature.SignerCertificate.Subject)"
if ($signature.TimeStamperCertificate) {
    Write-Output "タイムスタンプ署名者: $($signature.TimeStamperCertificate.Subject)"
}
