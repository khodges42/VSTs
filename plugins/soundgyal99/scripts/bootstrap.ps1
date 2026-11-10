# SPDX-License-Identifier: GPL-3.0-or-later
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot
$dependency=Join-Path $root '../../target/soundgyal-deps/JUCE'
$revision='4f43011b96eb0636104cb3e433894cda98243626'
if(!(Test-Path "$dependency/CMakeLists.txt")) {
 & git clone --depth 1 --branch 7.0.12 https://github.com/juce-framework/JUCE.git $dependency
 if($LASTEXITCODE){throw 'JUCE download failed'}
}
if(Test-Path "$dependency/.git") {
 $revisionFound=& git -c "safe.directory=$([IO.Path]::GetFullPath($dependency).Replace('\','/'))" -C $dependency rev-parse HEAD
 if($revisionFound -ne $revision){throw "Expected JUCE $revision; found $revisionFound"}
} else {
 & python "$PSScriptRoot/check_dependency.py" $dependency
 if($LASTEXITCODE){throw 'Bundled JUCE verification failed'}
}
Write-Output "JUCE GPLv3 source ready: $revision"
