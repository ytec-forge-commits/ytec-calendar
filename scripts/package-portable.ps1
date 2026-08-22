param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$releaseRoot = Join-Path $projectRoot "release"
$packageJson = Get-Content -LiteralPath (Join-Path $projectRoot "package.json") -Raw | ConvertFrom-Json
$version = $packageJson.version
$archivePath = Join-Path $releaseRoot "koyomado-v$version-windows-portable.zip"
$stagingPath = Join-Path $releaseRoot ".staging-koyomado-$PID"
$executablePath = Join-Path $projectRoot "src-tauri\target\release\koyomado.exe"

Push-Location $projectRoot
try {
    if (-not $SkipBuild) {
        npm run tauri:build
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
    Copy-Item -LiteralPath (Join-Path $projectRoot "PRIVACY.md") -Destination (Join-Path $stagingPath "PRIVACY.md")
    Copy-Item -LiteralPath (Join-Path $projectRoot "CODE_SIGNING_POLICY.md") -Destination (Join-Path $stagingPath "CODE_SIGNING_POLICY.md")
    Copy-Item -LiteralPath (Join-Path $projectRoot "src\assets\fonts\OFL.txt") -Destination (Join-Path $stagingPath "LINE_Seed_JP_OFL.txt")
    Copy-Item -LiteralPath (Join-Path $projectRoot "docs\Koyomado操作説明書.pdf") -Destination (Join-Path $stagingPath "Koyomado操作説明書.pdf")

    @(
        "このフォルダーに予定と設定が保存されます。"
        "アプリの更新や移動をするときも、このフォルダーを実行ファイルと一緒に残してください。"
    ) | Set-Content -LiteralPath (Join-Path $stagingPath "data\ここにデータが保存されます.txt") -Encoding UTF8

    Import-Module Microsoft.PowerShell.Security -ErrorAction Stop
    $signature = Get-AuthenticodeSignature -LiteralPath $executablePath
    $signatureNote = if ($signature.Status -eq "Valid") {
        "6. この実行ファイルはコード署名済みです。署名者とSHA-256を公式ページで確認してください。"
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
        "公式ページ: https://ytec.cloudfree.jp/ytb/koyomado/"
        "利用条件: LICENSE.txt / NOTICE"
        "プライバシー: PRIVACY.md"
        "コード署名方針: CODE_SIGNING_POLICY.md"
        "更新履歴: CHANGELOG.md"
        "第三者ライセンス: THIRD_PARTY_NOTICES.md / LINE_Seed_JP_OFL.txt"
    ) | Set-Content -LiteralPath (Join-Path $stagingPath "はじめに.txt") -Encoding UTF8

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
