param(
    [Parameter(Mandatory = $true)]
    [string]$Path,

    [string]$ExpectedVersion,
    [string]$ExpectedPackageName = "Y-TEC.Koyomado",
    [string]$ExpectedPublisher = "CN=F7BD381A-C29C-41A4-B039-8E9962198E21"
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "windows-sdk-tools.ps1")

$packagePath = (Resolve-Path -LiteralPath $Path).Path
$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("koyomado-msix-verify-" + [guid]::NewGuid().ToString("N"))
$unpackPath = Join-Path $temporaryRoot "unpacked"
$makeAppx = Get-LatestWindowsSdkTool -Name "makeappx.exe"

try {
    New-Item -ItemType Directory -Force -Path $unpackPath | Out-Null
    & $makeAppx unpack /o /p $packagePath /d $unpackPath
    if ($LASTEXITCODE -ne 0) { throw "MakeAppxによるMSIX展開検証に失敗しました。" }

    $manifestPath = Join-Path $unpackPath "AppxManifest.xml"
    $readerSettings = [System.Xml.XmlReaderSettings]::new()
    $readerSettings.DtdProcessing = [System.Xml.DtdProcessing]::Prohibit
    $readerSettings.XmlResolver = $null
    $reader = [System.Xml.XmlReader]::Create($manifestPath, $readerSettings)
    try {
        $manifest = [System.Xml.XmlDocument]::new()
        $manifest.XmlResolver = $null
        $manifest.Load($reader)
    }
    finally {
        $reader.Dispose()
    }

    $namespaceManager = [System.Xml.XmlNamespaceManager]::new($manifest.NameTable)
    $namespaceManager.AddNamespace("f", "http://schemas.microsoft.com/appx/manifest/foundation/windows10")
    $namespaceManager.AddNamespace("desktop", "http://schemas.microsoft.com/appx/manifest/desktop/windows10")
    $namespaceManager.AddNamespace("rescap", "http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities")

    $identity = $manifest.SelectSingleNode("/f:Package/f:Identity", $namespaceManager)
    if (-not $identity) { throw "MSIX manifestにIdentityがありません。" }
    if ($identity.GetAttribute("Name") -ne $ExpectedPackageName) { throw "Package Identity Nameが想定値と一致しません。" }
    if ($identity.GetAttribute("Publisher") -ne $ExpectedPublisher) { throw "Package Publisherが想定値と一致しません。" }
    if ($identity.GetAttribute("ProcessorArchitecture") -ne "x64") { throw "Package architectureがx64ではありません。" }
    if ($ExpectedVersion -and $identity.GetAttribute("Version") -ne $ExpectedVersion) { throw "Package versionが想定値と一致しません。" }

    $application = $manifest.SelectSingleNode("/f:Package/f:Applications/f:Application", $namespaceManager)
    if (-not $application -or $application.GetAttribute("Executable") -ne "app\koyomado.exe") {
        throw "Koyomadoのfull-trust実行ファイル定義を確認できません。"
    }
    if ($application.GetAttribute("EntryPoint") -ne "Windows.FullTrustApplication") {
        throw "KoyomadoのEntryPointがfull-trustではありません。"
    }

    $startupTask = $manifest.SelectSingleNode("//desktop:StartupTask[@TaskId='KoyomadoStartup']", $namespaceManager)
    if (-not $startupTask) { throw "KoyomadoStartupのStartupTask定義がありません。" }
    $runFullTrust = $manifest.SelectSingleNode("/f:Package/f:Capabilities/rescap:Capability[@Name='runFullTrust']", $namespaceManager)
    if (-not $runFullTrust) { throw "runFullTrust capabilityがありません。" }
    $unvirtualizedResources = $manifest.SelectSingleNode("/f:Package/f:Capabilities/rescap:Capability[@Name='unvirtualizedResources']", $namespaceManager)
    if ($unvirtualizedResources) {
        throw "Store版はMSIX LocalStateを使用するため、unvirtualizedResourcesを宣言しません。"
    }

    $requiredFiles = @(
        "app\koyomado.exe",
        "assets\StoreLogo.png",
        "assets\Square44x44Logo.png",
        "assets\Square150x150Logo.png",
        "legal\LICENSE.txt",
        "legal\NOTICE",
        "legal\THIRD_PARTY_NOTICES.md"
    )
    foreach ($requiredFile in $requiredFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $unpackPath $requiredFile) -PathType Leaf)) {
            throw "MSIXに必要なファイルがありません: $requiredFile"
        }
    }
    if (Test-Path -LiteralPath (Join-Path $unpackPath "data")) {
        throw "MSIXへ利用者データ用dataフォルダーを含めてはいけません。"
    }

    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $packagePath).Hash.ToLowerInvariant()
    Write-Output "MSIX構造を確認しました: $packagePath"
    Write-Output "Package version: $($identity.GetAttribute("Version"))"
    Write-Output "SHA-256: $hash"
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
