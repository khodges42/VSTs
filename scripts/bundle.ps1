param([switch]$DebugBuild,[ValidateSet('VIXEN','ECHOGIRL','EIS','SPASM')][string]$Plugin = 'VIXEN')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    $profile = if ($DebugBuild) { 'debug' } else { 'release' }
    $package = $Plugin.ToLowerInvariant()
    if ($DebugBuild) { cargo build -p $package --locked --offline } else { cargo build -p $package --release --locked --offline }
    if ($LASTEXITCODE -ne 0) { throw "$Plugin build failed" }
    $bundle = Join-Path $root "target/bundled/$Plugin.vst3"
    $contents = Join-Path $bundle 'Contents/x86_64-win'
    New-Item -ItemType Directory -Force $contents | Out-Null
    Copy-Item -LiteralPath (Join-Path $root "target/$profile/$package.dll") -Destination (Join-Path $contents "$Plugin.vst3") -Force
    $resources = Join-Path $bundle 'Contents/Resources'
    New-Item -ItemType Directory -Force $resources | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE'),(Join-Path $root 'NOTICE') -Destination $resources -Force
    $factory = Join-Path $resources 'factory'
    New-Item -ItemType Directory -Force $factory | Out-Null
    if (Test-Path -LiteralPath (Join-Path $root "factory/$package")) { Get-ChildItem -LiteralPath (Join-Path $root "factory/$package") -Filter "*.$package.json" | Copy-Item -Destination $factory -Force }
    python scripts/dependencies.py --collect (Join-Path $resources 'third-party')
    if ($LASTEXITCODE -ne 0) { throw 'Dependency inventory/license collection failed' }
    Write-Host "Built $bundle"
} finally { Pop-Location }
