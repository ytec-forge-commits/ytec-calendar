param(
    [switch]$SkipBuild,
    [string]$ExecutablePath
)

$ErrorActionPreference = "Stop"

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$releaseRoot = Join-Path $projectRoot "release"
$packageJson = Get-Content -LiteralPath (Join-Path $projectRoot "package.json") -Raw | ConvertFrom-Json
$version = $packageJson.version
$archivePath = Join-Path $releaseRoot "koyomado-v$version-windows-portable.zip"
$stagingPath = Join-Path $releaseRoot ".staging-koyomado-$PID"
$defaultExecutablePath = Join-Path $projectRoot "src-tauri\target\release\koyomado.exe"

Push-Location $projectRoot
try {
    if (-not $SkipBuild) {
        npm run tauri:build
        if ($LASTEXITCODE -ne 0) { throw "Tauriビルドに失敗しました。" }
    }
    $executablePath = if ($ExecutablePath) {
        (Resolve-Path -LiteralPath $ExecutablePath).Path
    }
    else {
        $defaultExecutablePath
    }
    if (-not (Test-Path -LiteralPath $executablePath -PathType Leaf)) {
        throw "ビルド済み実行ファイルが見つかりません: $executablePath"
    }

    New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null
    New-Item -ItemType Directory -Path $stagingPath | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $stagingPath "data") | Out-Null
    Copy-Item -LiteralPath $executablePath -Destination (Join-Path $stagingPath "koyomado.exe")
    Copy-Item -LiteralPath (Join-Path $projectRoot "LICENSE.txt") -Destination (Join-Path $stagingPath "LICENSE.txt")
    Copy-Item -LiteralPath (Join-Path $projectRoot "NOTICE") -Destination (Join-Path $stagingPath "NOTICE")
    Copy-Item -LiteralPath (Join-Path $projectRoot "CHANGELOG.md") -Destination (Join-Path $stagingPath "CHANGELOG.md")
    Copy-Item -LiteralPath (Join-Path $projectRoot "THIRD_PARTY_NOTICES.md") -Destination (Join-Path $stagingPath "THIRD_PARTY_NOTICES.md")
    Copy-Item -LiteralPath (Join-Path $projectRoot "third_party\opengameart-cc0-notification-sounds\NOTICE.txt") -Destination (Join-Path $stagingPath "NOTIFICATION_SOUNDS_CC0.txt")
    Copy-Item -LiteralPath (Join-Path $projectRoot "PRIVACY.md") -Destination (Join-Path $stagingPath "PRIVACY.md")
    Copy-Item -LiteralPath (Join-Path $projectRoot "CODE_SIGNING_POLICY.md") -Destination (Join-Path $stagingPath "CODE_SIGNING_POLICY.md")
    Copy-Item -LiteralPath (Join-Path $projectRoot "src\assets\fonts\OFL.txt") -Destination (Join-Path $stagingPath "LINE_Seed_JP_OFL.txt")
    Copy-Item -LiteralPath (Join-Path $projectRoot "docs\Koyomado操作説明書.pdf") -Destination (Join-Path $stagingPath "Koyomado操作説明書.pdf")
    Copy-Item -LiteralPath (Join-Path $projectRoot "docs\Koyomado.en.pdf") -Destination (Join-Path $stagingPath "Koyomado.en.pdf")
    Copy-Item -LiteralPath (Join-Path $projectRoot "ASSET_PROVENANCE.md") -Destination (Join-Path $stagingPath "ASSET_PROVENANCE.md")
    New-Item -ItemType Directory -Path (Join-Path $stagingPath "docs") | Out-Null
    Copy-Item -LiteralPath (Join-Path $projectRoot "docs\asset-manifest.json") -Destination (Join-Path $stagingPath "docs\asset-manifest.json")
    New-Item -ItemType Directory -Path (Join-Path $stagingPath "third_party") | Out-Null
    foreach ($noticeDirectory in @("dependency-licenses", "mpl-source")) {
        Copy-Item -LiteralPath (Join-Path $projectRoot "third_party\$noticeDirectory") -Destination (Join-Path $stagingPath "third_party") -Recurse
    }

    @(
        "このフォルダーに予定と設定が保存されます。"
        "アプリの更新や移動をするときも、このフォルダーを実行ファイルと一緒に残してください。"
    ) | Set-Content -LiteralPath (Join-Path $stagingPath "data\ここにデータが保存されます.txt") -Encoding UTF8

    Import-Module Microsoft.PowerShell.Security -ErrorAction Stop
    $signature = Get-AuthenticodeSignature -LiteralPath $executablePath
    $signatureNote = if ($signature.SignerCertificate) {
        "6. この実行ファイルはコード署名済みです。自己署名版ではWindowsの警告が表示される場合があります。署名者とSHA-256を公式ページで確認してください。"
    }
    else {
        "6. この実行ファイルは未署名です。公式ページから入手し、掲載されたSHA-256を確認してください。"
    }

    @(
        "Koyomado $version"
        ""
        "1. koyomado.exe を起動してください。"
        "2. 配置場所を決めた後、右上の歯車からWindows自動起動をONにできます。"
        "3. 予定と設定は同じ場所の data フォルダーへ保存されます。"
        "4. 更新時は data フォルダーを残し、実行ファイルと同梱文書を差し替えてください。"
        "5. Google Drive上では複数PCから同時に起動しないでください。"
        $signatureNote
        ""
        "操作説明書: Koyomado操作説明書.pdf"
        "英語説明書: Koyomado.en.pdf（アプリのUIは日本語です）"
        "公式ページ: https://ytec.cloudfree.jp/forge/projects/koyomado/"
        "お問い合わせ: https://ytec.cloudfree.jp/forge/contact/"
        "利用条件: LICENSE.txt / NOTICE"
        "プライバシー: PRIVACY.md"
        "コード署名方針: CODE_SIGNING_POLICY.md"
        "更新履歴: CHANGELOG.md"
        "第三者ライセンス: THIRD_PARTY_NOTICES.md / third_party / LINE_Seed_JP_OFL.txt / NOTIFICATION_SOUNDS_CC0.txt"
    ) | Set-Content -LiteralPath (Join-Path $stagingPath "はじめに.txt") -Encoding UTF8

    @(
        "Koyomado $version - Portable edition"
        ""
        "The application interface is Japanese. English instructions: Koyomado.en.pdf"
        "Extract the entire ZIP into a writable folder, then run koyomado.exe."
        "Events and settings are stored in the adjacent data folder."
        "Before updating, quit Koyomado and back up data. Retain your existing data folder; do not replace it with this empty distribution folder."
        "Do not run the same Google Drive folder on multiple PCs simultaneously."
        "Google integration is optional and initially OFF. Never send OAuth JSON, tokens or actual events to support."
        "Formal portable distributions use Y-TEC self-signing and SHA-256. Self-signing does not remove all Windows warnings. An unsigned local test ZIP is not an official release."
        ""
        "Product page: https://ytec.cloudfree.jp/forge/en/projects/koyomado/"
        "Contact: https://ytec.cloudfree.jp/forge/contact/"
        "License and notices: LICENSE.txt / NOTICE / THIRD_PARTY_NOTICES.md / third_party"
        "Privacy: PRIVACY.md"
    ) | Set-Content -LiteralPath (Join-Path $stagingPath "README.en.txt") -Encoding UTF8

    if (Test-Path -LiteralPath $archivePath) {
        Remove-Item -LiteralPath $archivePath -Force
    }
    Compress-Archive -Path (Join-Path $stagingPath "*") -DestinationPath $archivePath -CompressionLevel Optimal
    Write-Output "ポータブルZIPを作成しました: $archivePath"
}
finally {
    Pop-Location
    if (Test-Path -LiteralPath $stagingPath) {
        $resolvedRelease = [System.IO.Path]::GetFullPath($releaseRoot).TrimEnd('\') + '\'
        $resolvedStaging = [System.IO.Path]::GetFullPath($stagingPath)
        if (-not $resolvedStaging.StartsWith($resolvedRelease, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "一時フォルダーがrelease外を指しているため削除を中止しました: $resolvedStaging"
        }
        Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
    }
}
