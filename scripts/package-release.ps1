param(
    [string]$OutputDirectory = "dist"
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$outputPath = if ([System.IO.Path]::IsPathRooted($OutputDirectory)) {
    $OutputDirectory
} else {
    Join-Path $projectRoot $OutputDirectory
}
$packageDirectory = Join-Path $outputPath "which-bowl-win64"
$zipPath = Join-Path $outputPath "which-bowl-win64.zip"
$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
$cargoPath = if ($null -ne $cargoCommand) {
    $cargoCommand.Source
} else {
    Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
}

if (-not (Test-Path -LiteralPath $cargoPath)) {
    throw "Cargo was not found. Install Rust or add Cargo to PATH before packaging."
}

Push-Location $projectRoot
try {
    $vcvarsPath = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
    if (Test-Path -LiteralPath $vcvarsPath) {
        $buildCommand = "call `"$vcvarsPath`" >nul && `"$cargoPath`" build --release"
        & cmd.exe /d /s /c $buildCommand
    } else {
        & $cargoPath build --release
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed."
    }

    New-Item -ItemType Directory -Force -Path $outputPath | Out-Null
    if (Test-Path -LiteralPath $packageDirectory) {
        Remove-Item -LiteralPath $packageDirectory -Recurse -Force
    }
    if (Test-Path -LiteralPath $zipPath) {
        Remove-Item -LiteralPath $zipPath -Force
    }

    New-Item -ItemType Directory -Path $packageDirectory | Out-Null
    Copy-Item -LiteralPath (Join-Path $projectRoot "target\release\which-bowl.exe") -Destination $packageDirectory
    Copy-Item -LiteralPath (Join-Path $projectRoot "target\release\assets") -Destination (Join-Path $packageDirectory "assets") -Recurse
    Compress-Archive -LiteralPath $packageDirectory -DestinationPath $zipPath

    Write-Host "Package created: $zipPath"
    Write-Host "To play without unzipping, run: $packageDirectory\which-bowl.exe"
} finally {
    Pop-Location
}
