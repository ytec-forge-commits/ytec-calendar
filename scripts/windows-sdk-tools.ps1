function Get-LatestWindowsSdkTool {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet("makeappx.exe", "signtool.exe", "appcert.exe")]
        [string]$Name
    )

    if ($Name -eq "appcert.exe") {
        $appCertificationKitPaths = @(
            (Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\App Certification Kit\appcert.exe"),
            (Join-Path $env:ProgramFiles "Windows Kits\10\App Certification Kit\appcert.exe")
        ) | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) }

        $appCertificationKit = $appCertificationKitPaths | Select-Object -First 1
        if ($appCertificationKit) {
            return $appCertificationKit
        }
    }

    $roots = @(
        (Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"),
        (Join-Path $env:ProgramFiles "Windows Kits\10\bin")
    ) | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Container) }

    $candidates = foreach ($root in $roots) {
        Get-ChildItem -LiteralPath $root -Directory -ErrorAction SilentlyContinue |
            ForEach-Object {
                $candidate = Join-Path $_.FullName "x64\$Name"
                if (Test-Path -LiteralPath $candidate -PathType Leaf) {
                    try {
                        $sdkVersion = [version]$_.Name
                    }
                    catch {
                        $sdkVersion = [version]"0.0"
                    }
                    [pscustomobject]@{
                        Version = $sdkVersion
                        Path = $candidate
                    }
                }
            }
    }

    $tool = $candidates | Sort-Object Version -Descending | Select-Object -First 1
    if (-not $tool) {
        throw "$Name が見つかりません。Windows SDKをインストールしてください。"
    }
    return $tool.Path
}
