param(
    [switch]$SkipArtifactHashCheck
)

$ErrorActionPreference = "Stop"
Import-Module Microsoft.PowerShell.Utility -ErrorAction Stop
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$draftPath = Join-Path $projectRoot "docs\release\final-strict\partner-center-submission-draft.md"
$draft = Get-Content -LiteralPath $draftPath -Raw -Encoding UTF8
$packageJson = Get-Content -LiteralPath (Join-Path $projectRoot "package.json") -Raw -Encoding UTF8 | ConvertFrom-Json
$appVersion = [string]$packageJson.version

$searchTermsMatch = [regex]::Match(
    $draft,
    '(?ms)^### Search terms\s+`(?<terms>[^`]+)`'
)
if (-not $searchTermsMatch.Success) {
    throw "Store search terms were not found in the Partner Center draft."
}
$searchTerms = @($searchTermsMatch.Groups["terms"].Value.Split(',') | ForEach-Object {
    $_.Trim()
} | Where-Object { $_ })
if ($searchTerms.Count -gt 7) {
    throw "Microsoft Store permits at most seven search terms; found $($searchTerms.Count)."
}

$expectedPrivacyUrl = "https://github.com/ytec-forge-commits/ytec-calendar/blob/main/PRIVACY.md"
if (-not $draft.Contains("Privacy policy: ``$expectedPrivacyUrl``")) {
    throw "The Partner Center draft does not use the Koyomado-specific privacy policy URL."
}

$expectedUploadRelativePath = "release/koyomado-v$appVersion-store-x64.msixupload"
$uploadPath = Join-Path $projectRoot ($expectedUploadRelativePath -replace '/', '\')
if (-not $draft.Contains("Upload file: ``$expectedUploadRelativePath``")) {
    throw "The Partner Center draft does not point to the expected Store upload artifact."
}
$uploadHashMatch = [regex]::Match(
    $draft,
    '(?ms)^## Store package\s+.*?^- SHA-256: `(?<hash>[0-9a-fA-F]{64})`'
)
if (-not $uploadHashMatch.Success) {
    throw "The Store upload SHA-256 was not found in the Partner Center draft."
}
if (-not $SkipArtifactHashCheck) {
    if (-not (Test-Path -LiteralPath $uploadPath -PathType Leaf)) {
        throw "The Store upload artifact is missing: $expectedUploadRelativePath"
    }
    $actualUploadHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $uploadPath).Hash.ToLowerInvariant()
    $documentedUploadHash = $uploadHashMatch.Groups["hash"].Value.ToLowerInvariant()
    if ($documentedUploadHash -ne $actualUploadHash) {
        throw "The Partner Center draft Store upload SHA-256 does not match the current artifact."
    }
}
if (-not $draft.Contains('Restricted capability justification for Submission options:')) {
    throw "The runFullTrust restricted-capability justification is missing."
}
if (-not $draft.Contains('`runFullTrust`')) {
    throw "The runFullTrust capability name is missing from the submission notes."
}

Add-Type -AssemblyName System.Drawing
$screenshots = @(
    "koyomado-store-calendar-ja.png",
    "koyomado-store-event-editor-ja.png",
    "koyomado-store-day-agenda-ja.png",
    "koyomado-store-moon-theme-ja.png"
)
foreach ($name in $screenshots) {
    $path = Join-Path $projectRoot "docs\release\final-strict\store-assets\$name"
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required Store screenshot is missing: $name"
    }
    $image = [System.Drawing.Image]::FromFile($path)
    try {
        if ($image.Width -lt 1366 -or $image.Height -lt 768) {
            throw "Store screenshot is smaller than 1366x768: $name"
        }
    }
    finally {
        $image.Dispose()
    }
}

Write-Output "partner-center-submission tests: PASS ($($searchTerms.Count) search terms, $($screenshots.Count) screenshots)"
