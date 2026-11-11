# Optional initial installation only. Nothing is installed by staging packages.
[CmdletBinding(SupportsShouldProcess=$true)]
param(
    [Parameter(Mandatory=$true)][string]$Destination,
    [ValidateSet('All','vixen','echogirl','eis','spasm','soundgyal99','dubmist98')]
    [string]$Plugin='All'
)
$ErrorActionPreference='Stop'
$preview=$WhatIfPreference
# Read-only validation must still run when the eventual copies are previewed.
$WhatIfPreference=$false
$stage=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'windows-x64'))
$records=Get-Content -LiteralPath (Join-Path $stage 'current.json') -Raw | ConvertFrom-Json
$chosen=@($records | Where-Object {$Plugin -eq 'All' -or $_.plugin -eq $Plugin})
if(!$chosen.Count){throw 'No staged package matches this plugin. Run scripts/stage_releases.py first.'}
$destinationRoot=[IO.Path]::GetFullPath($Destination).TrimEnd('\','/')
foreach($entry in $chosen){
    $source=[IO.Path]::GetFullPath((Join-Path "$stage/VST3" $entry.bundle))
    if((Split-Path -Parent $source) -ne [IO.Path]::GetFullPath("$stage/VST3")){throw 'Invalid bundle path'}
    $target=[IO.Path]::GetFullPath((Join-Path $destinationRoot $entry.bundle))
    if((Split-Path -Parent $target) -ne $destinationRoot){throw 'Invalid install target'}
    if(Test-Path -LiteralPath $target){throw "Already exists: $target. Close the DAW and replace the complete folder manually for an update."}
    $binary=Join-Path $source "Contents/x86_64-win/$($entry.bundle)"
    if((Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.binary_sha256){throw "Staged binary checksum differs: $binary"}
}
$WhatIfPreference=$preview
foreach($entry in $chosen){
    $source=Join-Path "$stage/VST3" $entry.bundle
    if($PSCmdlet.ShouldProcess((Join-Path $destinationRoot $entry.bundle),'Copy complete VST3 bundle')){
        New-Item -ItemType Directory -Force -Path $destinationRoot | Out-Null
        Copy-Item -LiteralPath $source -Destination $destinationRoot -Recurse
    }
}
