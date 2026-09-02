param(
    [Parameter(Mandatory = $true)]
    [string]$Path,

    [Parameter(Mandatory = $true)]
    [string]$CertificateThumbprint,

    [string]$ExpectedSubject,

    [string]$TimestampUrl,

    [switch]$RequireSelfSignedNonExportable
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "windows-sdk-tools.ps1")
. (Join-Path $PSScriptRoot "code-signing-certificate-policy.ps1")

$resolvedPath = (Resolve-Path -LiteralPath $Path).Path
$thumbprint = ($CertificateThumbprint -replace "\s", "").ToUpperInvariant()
$certificate = Get-ChildItem -LiteralPath "Cert:\CurrentUser\My" |
    Where-Object { $_.Thumbprint -eq $thumbprint } |
    Select-Object -First 1

if (-not $certificate) {
    throw "指定された証明書がCurrentUser\\Myに見つかりません。"
}
if (-not $certificate.HasPrivateKey) {
    throw "指定された証明書には署名用の秘密鍵がありません。"
}
if ($certificate.NotBefore -gt (Get-Date) -or $certificate.NotAfter -le (Get-Date)) {
    throw "指定された証明書は現在有効ではありません。"
}
if ($ExpectedSubject -and $certificate.Subject -ne $ExpectedSubject) {
    throw "証明書Subjectが想定値と一致しません。"
}
$codeSigningOid = "1.3.6.1.5.5.7.3.3"
$enhancedKeyUsageOids = @($certificate.EnhancedKeyUsageList | ForEach-Object {
    if ($_.ObjectId -is [System.Security.Cryptography.Oid]) {
        $_.ObjectId.Value
    }
    else {
        [string]$_.ObjectId
    }
})
if ($codeSigningOid -notin $enhancedKeyUsageOids) {
    throw "指定された証明書にはコード署名用途がありません。"
}
if ($RequireSelfSignedNonExportable) {
    $null = Assert-SelfSignedNonExportableCodeSigningCertificate `
        -Certificate $certificate `
        -ExpectedSubject $ExpectedSubject
}

$signTool = Get-LatestWindowsSdkTool -Name "signtool.exe"
$arguments = @("sign", "/fd", "SHA256", "/sha1", $thumbprint, "/s", "My")
if ($TimestampUrl) {
    $arguments += @("/tr", $TimestampUrl, "/td", "SHA256")
}
$arguments += $resolvedPath

& $signTool @arguments
if ($LASTEXITCODE -ne 0) {
    throw "SignToolによる署名に失敗しました。"
}

$signature = Get-AuthenticodeSignature -LiteralPath $resolvedPath
if (-not $signature.SignerCertificate -or $signature.SignerCertificate.Thumbprint -ne $thumbprint) {
    throw "署名後のファイルから想定した署名者を確認できません。"
}

Write-Output "署名しました: $resolvedPath"
Write-Output "署名者: $($certificate.Subject)"
