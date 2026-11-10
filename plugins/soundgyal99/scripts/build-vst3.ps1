# SPDX-License-Identifier: GPL-3.0-or-later
param([switch]$SkipTests)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot
& "$PSScriptRoot/bootstrap.ps1"
$vswhere="${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(!$vs){throw 'Install Visual Studio C++ x64 Build Tools with CMake and Windows SDK'}
$cmake=Join-Path $vs 'Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin/cmake.exe'
if(!(Test-Path $cmake)){$cmake=(Get-Command cmake -ErrorAction Stop).Source}
$out=[IO.Path]::GetFullPath((Join-Path $root '../../target/soundgyal-cmake'))
# CMake chooses the newest installed Visual Studio; platform is explicitly x64.
& $cmake -S $root -B $out -A x64
if($LASTEXITCODE){throw 'Configure failed'}
& $cmake --build $out --config Release --parallel 4
if($LASTEXITCODE){throw 'Build failed'}
if(!$SkipTests){
 $ctest=Join-Path (Split-Path $cmake) 'ctest.exe'
 & $ctest --test-dir $out -C Release --output-on-failure
 if($LASTEXITCODE){throw 'Tests failed'}
 $bundle=Join-Path $out 'Soundgyal99_artefacts/Release/VST3/SOUNDGYAL 99.vst3'
 & "$out/soundgyal_host_artefacts/Release/soundgyal_host.exe" $bundle "$out/validation/soundgyal-vst3.png"
 if($LASTEXITCODE){throw 'Actual VST3 smoke test failed'}
}
Write-Output "VST3 bundle: $out/Soundgyal99_artefacts/Release/VST3/SOUNDGYAL 99.vst3"
