# MSVC x64 optimized offline milestone build
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot
$out=Join-Path $root '../../target/soundgyal99'
New-Item -ItemType Directory -Force $out | Out-Null
$vswhere="${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(!$vs){throw 'MSVC x64 tools required'}
$vcvars=Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
$sources=@(Get-ChildItem "$root/vendor/plaits/dsp" -Recurse -Filter *.cc | ForEach-Object FullName)
$sources+=@("$root/vendor/plaits/resources.cc","$root/vendor/stmlib/dsp/units.cc","$root/vendor/stmlib/dsp/atan.cc","$root/vendor/stmlib/utils/random.cc","$root/src/engine.cpp","$root/tests/validate.cpp")
$argsList=@('/nologo','/std:c++17','/EHsc','/O2','/DTEST','/D_USE_MATH_DEFINES','/permissive-','/FIcstdio',('/I"'+$root+'/vendor"'),('/I"'+$root+'/src"'),'/Fe:soundgyal_validate.exe')
$argsList+= $sources | ForEach-Object {'"'+$_+'"'}
$rsp=Join-Path $out 'compile.rsp'
$argsList | Set-Content $rsp
Push-Location $out
try {
 & cmd.exe /d /c "`"$vcvars`" >nul && cl.exe @compile.rsp >build.log 2>&1"
 if($LASTEXITCODE){Get-Content build.log -Tail 60;throw 'Compile failed'}
 & ./soundgyal_validate.exe
 if($LASTEXITCODE){throw 'Validation failed'}
} finally {Pop-Location}
