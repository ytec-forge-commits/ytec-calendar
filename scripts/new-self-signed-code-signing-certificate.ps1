param(
    [string]$Subject = "CN=Y-TEC",
    [string]$FriendlyName = "Y-TEC Self-Signed Code Signing",
    [int]$ValidYears = 3
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "code-signing-certificate-policy.ps1")
if ($ValidYears -lt 1 -or $ValidYears -gt 5) {
    throw "有効期間は1～5年で指定してください。"
}

$existing = Get-ChildItem -LiteralPath "Cert:\CurrentUser\My" |
    Where-Object {
        $_.Subject -eq $Subject -and
        $_.FriendlyName -eq $FriendlyName -and
        $_.NotAfter -gt (Get-Date).AddDays(30) -and
        (Test-SelfSignedNonExportableCodeSigningCertificate `
            -Certificate $_ `
            -ExpectedSubject $Subject)
    } |
    Sort-Object NotAfter -Descending |
    Select-Object -First 1

if ($existing) {
    Write-Output "既存の証明書を使用します。"
    Write-Output "Thumbprint: $($existing.Thumbprint)"
    return
}

$certificate = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $Subject `
    -FriendlyName $FriendlyName `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -HashAlgorithm "SHA256" `
    -KeyAlgorithm "RSA" `
    -KeyLength 3072 `
    -KeyExportPolicy NonExportable `
    -NotAfter (Get-Date).AddYears($ValidYears)

$null = Assert-SelfSignedNonExportableCodeSigningCertificate `
    -Certificate $certificate `
    -ExpectedSubject $Subject

Write-Output "自己署名コード署名証明書をCurrentUser\\Myへ作成しました。秘密鍵は非エクスポートです。"
Write-Output "Thumbprint: $($certificate.Thumbprint)"
