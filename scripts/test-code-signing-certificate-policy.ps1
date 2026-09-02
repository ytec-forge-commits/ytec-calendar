param()

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "code-signing-certificate-policy.ps1")

function Assert-Equal {
    param(
        [Parameter(Mandatory = $true)]
        [bool]$Expected,
        [Parameter(Mandatory = $true)]
        [bool]$Actual,
        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    if ($Expected -ne $Actual) {
        throw "$Message (expected=$Expected, actual=$Actual)"
    }
}

$valid = @{
    Subject = "CN=Y-TEC"
    Issuer = "CN=Y-TEC"
    HasPrivateKey = $true
    EnhancedKeyUsageOids = @("1.3.6.1.5.5.7.3.3")
    PrivateKeyExportable = $false
}

Assert-Equal -Expected $true -Actual (Test-CodeSigningCertificateMetadata @valid) `
    -Message "非エクスポートの自己署名コード署名証明書を許可する必要があります。"

$wrongIssuer = $valid.Clone()
$wrongIssuer.Issuer = "CN=Different Issuer"
Assert-Equal -Expected $false -Actual (Test-CodeSigningCertificateMetadata @wrongIssuer) `
    -Message "自己署名ではない証明書を拒否する必要があります。"

$wrongEku = $valid.Clone()
$wrongEku.EnhancedKeyUsageOids = @("1.3.6.1.5.5.7.3.1")
Assert-Equal -Expected $false -Actual (Test-CodeSigningCertificateMetadata @wrongEku) `
    -Message "コード署名EKUがない証明書を拒否する必要があります。"

$exportable = $valid.Clone()
$exportable.PrivateKeyExportable = $true
Assert-Equal -Expected $false -Actual (Test-CodeSigningCertificateMetadata @exportable) `
    -Message "エクスポート可能な秘密鍵を拒否する必要があります。"

$noPrivateKey = $valid.Clone()
$noPrivateKey.HasPrivateKey = $false
Assert-Equal -Expected $false -Actual (Test-CodeSigningCertificateMetadata @noPrivateKey) `
    -Message "秘密鍵がない証明書を拒否する必要があります。"

Write-Output "code-signing-certificate-policy tests: PASS"
