param(
    [switch]$SkipBuild,
    [string]$PackageVersion,
    [string]$CertificateThumbprint,
    [switch]$CreateUpload
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "windows-sdk-tools.ps1")

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$releaseRoot = Join-Path $projectRoot "release"
$packageJson = Get-Content -LiteralPath (Join-Path $projectRoot "package.json") -Raw | ConvertFrom-Json
$appVersion = [string]$packageJson.version

if (-not $PackageVersion) {
    if ($appVersion -notmatch '^(?<major>[1-9]\d*)\.(?<minor>\d+)\.(?<patch>\d+)$') {
        throw "プレリリース版からStore用versionを推測しません。-PackageVersion（例: 1.0.0.0）を指定してください。"
    }
    $PackageVersion = "$($Matches.major).$($Matches.minor).$($Matches.patch).0"
}
if ($PackageVersion -notmatch '^[1-9]\d{0,4}\.(\d{1,5})\.(\d{1,5})\.0$') {
    throw "MSIX versionは1～65535のmajorと0～65535のminor/build、第四区分0で指定してください。"
}
$versionParts = $PackageVersion.Split('.') | ForEach-Object { [int]$_ }
if ($versionParts | Where-Object { $_ -gt 65535 }) {
    throw "MSIX versionの各区分は65535以下にしてください。"
}

$channel = if ($CertificateThumbprint) { "local-test" } else { "store" }
$msixPath = Join-Path $releaseRoot "koyomado-v$appVersion-$channel-x64.msix"
$uploadPath = Join-Path $releaseRoot "koyomado-v$appVersion-store-x64.msixupload"
$stagingPath = Join-Path $releaseRoot ".staging-msix-$PID"
$uploadStagingPath = Join-Path $releaseRoot ".staging-msixupload-$PID"
$executablePath = Join-Path $projectRoot "src-tauri\target\release\koyomado.exe"
$manifestTemplate = Join-Path $projectRoot "packaging\msix\AppxManifest.xml.in"
$iconsPath = Join-Path $projectRoot "src-tauri\icons"
$makeAppx = Get-LatestWindowsSdkTool -Name "makeappx.exe"

Push-Location $projectRoot
try {
    if (-not $SkipBuild) {
        npm run tauri:build
        if ($LASTEXITCODE -ne 0) { throw "Tauriビルドに失敗しました。" }
    }
    if (-not (Test-Path -LiteralPath $executablePath -PathType Leaf)) {
        throw "ビルド済み実行ファイルが見つかりません: $executablePath"
    }

    New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $stagingPath "app") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $stagingPath "assets") -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $stagingPath "legal") -Force | Out-Null
    Copy-Item -LiteralPath $executablePath -Destination (Join-Path $stagingPath "app\koyomado.exe")

    $assetNames = @(
        "StoreLogo.png",
        "Square44x44Logo.png",
        "Square71x71Logo.png",
        "Square150x150Logo.png"
    )
    foreach ($assetName in $assetNames) {
        Copy-Item -LiteralPath (Join-Path $iconsPath $assetName) -Destination (Join-Path $stagingPath "assets\$assetName")
    }
    $legalFiles = @(
        "LICENSE.txt",
        "NOTICE",
        "THIRD_PARTY_NOTICES.md",
        "PRIVACY.md",
        "CODE_SIGNING_POLICY.md"
    )
    foreach ($legalFile in $legalFiles) {
        Copy-Item -LiteralPath (Join-Path $projectRoot $legalFile) -Destination (Join-Path $stagingPath "legal\$legalFile")
    }
    Copy-Item -LiteralPath (Join-Path $projectRoot "src\assets\fonts\OFL.txt") -Destination (Join-Path $stagingPath "legal\LINE_Seed_JP_OFL.txt")
    Copy-Item -LiteralPath (Join-Path $projectRoot "third_party\opengameart-cc0-notification-sounds\NOTICE.txt") -Destination (Join-Path $stagingPath "legal\NOTIFICATION_SOUNDS_CC0.txt")

    $manifest = [System.IO.File]::ReadAllText(
        $manifestTemplate,
        [System.Text.UTF8Encoding]::new($false)
    ).Replace("__PACKAGE_VERSION__", $PackageVersion)
    [System.IO.File]::WriteAllText(
        (Join-Path $stagingPath "AppxManifest.xml"),
        $manifest,
        [System.Text.UTF8Encoding]::new($false)
    )

    if (Test-Path -LiteralPath $msixPath) { Remove-Item -LiteralPath $msixPath -Force }
    & $makeAppx pack /o /h SHA256 /d $stagingPath /p $msixPath
    if ($LASTEXITCODE -ne 0) { throw "MakeAppxによるMSIX作成に失敗しました。" }

    if ($CertificateThumbprint) {
        & (Join-Path $PSScriptRoot "sign-windows-artifact.ps1") `
            -Path $msixPath `
            -CertificateThumbprint $CertificateThumbprint `
            -ExpectedSubject "CN=F7BD381A-C29C-41A4-B039-8E9962198E21" `
            -RequireSelfSignedNonExportable
    }

    & (Join-Path $PSScriptRoot "verify-msix-package.ps1") `
        -Path $msixPath `
        -ExpectedVersion $PackageVersion

    if ($CreateUpload -and -not $CertificateThumbprint) {
        New-Item -ItemType Directory -Path $uploadStagingPath -Force | Out-Null
        Copy-Item -LiteralPath $msixPath -Destination (Join-Path $uploadStagingPath (Split-Path -Leaf $msixPath))
        $temporaryZip = [System.IO.Path]::ChangeExtension($uploadPath, ".zip")
        if (Test-Path -LiteralPath $temporaryZip) { Remove-Item -LiteralPath $temporaryZip -Force }
        if (Test-Path -LiteralPath $uploadPath) { Remove-Item -LiteralPath $uploadPath -Force }
        Compress-Archive -Path (Join-Path $uploadStagingPath "*") -DestinationPath $temporaryZip -CompressionLevel Optimal
        Move-Item -LiteralPath $temporaryZip -Destination $uploadPath
        Write-Output "Store提出用uploadを作成しました: $uploadPath"
    }

    Write-Output "MSIXを作成しました: $msixPath"
    Write-Output "Package version: $PackageVersion"
}
finally {
    Pop-Location
    foreach ($temporaryPath in @($stagingPath, $uploadStagingPath)) {
        if (Test-Path -LiteralPath $temporaryPath) {
            $resolvedRelease = [System.IO.Path]::GetFullPath($releaseRoot).TrimEnd('\') + '\'
            $resolvedTemporary = [System.IO.Path]::GetFullPath($temporaryPath)
            if (-not $resolvedTemporary.StartsWith($resolvedRelease, [System.StringComparison]::OrdinalIgnoreCase)) {
                throw "一時フォルダーがrelease外を指しているため削除を中止しました: $resolvedTemporary"
            }
            Remove-Item -LiteralPath $resolvedTemporary -Recurse -Force
        }
    }
}
