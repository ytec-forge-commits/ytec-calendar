function Test-CodeSigningCertificateMetadata {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Subject,
        [Parameter(Mandatory = $true)]
        [string]$Issuer,
        [Parameter(Mandatory = $true)]
        [bool]$HasPrivateKey,
        [Parameter(Mandatory = $true)]
        [string[]]$EnhancedKeyUsageOids,
        [Parameter(Mandatory = $true)]
        [bool]$PrivateKeyExportable
    )

    $codeSigningOid = "1.3.6.1.5.5.7.3.3"
    return $HasPrivateKey -and
        $Subject -eq $Issuer -and
        $codeSigningOid -in $EnhancedKeyUsageOids -and
        -not $PrivateKeyExportable
}

function Get-CodeSigningCertificateEkuOids {
    param(
        [Parameter(Mandatory = $true)]
        [System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate
    )

    return @($Certificate.EnhancedKeyUsageList | ForEach-Object {
        if ($_.ObjectId -is [System.Security.Cryptography.Oid]) {
            $_.ObjectId.Value
        }
        else {
            [string]$_.ObjectId
        }
    })
}

function Get-CodeSigningPrivateKeyExportable {
    param(
        [Parameter(Mandatory = $true)]
        [System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate
    )

    if (-not $Certificate.HasPrivateKey) {
        return $null
    }

    $rsa = [System.Security.Cryptography.X509Certificates.RSACertificateExtensions]::GetRSAPrivateKey($Certificate)
    if (-not $rsa) {
        return $null
    }

    try {
        if ($rsa -is [System.Security.Cryptography.RSACng]) {
            $exportPolicy = $rsa.Key.ExportPolicy
            $exportFlags = [System.Security.Cryptography.CngExportPolicies]::AllowExport -bor
                [System.Security.Cryptography.CngExportPolicies]::AllowPlaintextExport -bor
                [System.Security.Cryptography.CngExportPolicies]::AllowArchiving -bor
                [System.Security.Cryptography.CngExportPolicies]::AllowPlaintextArchiving
            return ($exportPolicy -band $exportFlags) -ne 0
        }

        if ($rsa -is [System.Security.Cryptography.RSACryptoServiceProvider]) {
            return $rsa.CspKeyContainerInfo.Exportable
        }

        return $null
    }
    finally {
        $rsa.Dispose()
    }
}

function Assert-SelfSignedNonExportableCodeSigningCertificate {
    param(
        [Parameter(Mandatory = $true)]
        [System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate,
        [string]$ExpectedSubject
    )

    if ($ExpectedSubject -and $Certificate.Subject -ne $ExpectedSubject) {
        throw "証明書Subjectが想定値と一致しません。"
    }
    if ($Certificate.Subject -ne $Certificate.Issuer) {
        throw "直接配布には自己署名証明書だけを使用できます。"
    }
    if (-not $Certificate.HasPrivateKey) {
        throw "指定された証明書には署名用の秘密鍵がありません。"
    }

    $enhancedKeyUsageOids = Get-CodeSigningCertificateEkuOids -Certificate $Certificate
    if ("1.3.6.1.5.5.7.3.3" -notin $enhancedKeyUsageOids) {
        throw "指定された証明書にはコード署名用途がありません。"
    }

    $privateKeyExportable = Get-CodeSigningPrivateKeyExportable -Certificate $Certificate
    if ($null -eq $privateKeyExportable) {
        throw "秘密鍵のエクスポート可否を安全に確認できません。"
    }
    if ($privateKeyExportable) {
        throw "エクスポート可能な秘密鍵は直接配布の署名に使用できません。"
    }

    if (-not (Test-CodeSigningCertificateMetadata `
        -Subject $Certificate.Subject `
        -Issuer $Certificate.Issuer `
        -HasPrivateKey $Certificate.HasPrivateKey `
        -EnhancedKeyUsageOids $enhancedKeyUsageOids `
        -PrivateKeyExportable $privateKeyExportable)) {
        throw "証明書が自己署名コード署名ポリシーを満たしていません。"
    }

    return $Certificate
}

function Test-SelfSignedNonExportableCodeSigningCertificate {
    param(
        [Parameter(Mandatory = $true)]
        [System.Security.Cryptography.X509Certificates.X509Certificate2]$Certificate,
        [string]$ExpectedSubject
    )

    try {
        $null = Assert-SelfSignedNonExportableCodeSigningCertificate `
            -Certificate $Certificate `
            -ExpectedSubject $ExpectedSubject
        return $true
    }
    catch {
        return $false
    }
}
