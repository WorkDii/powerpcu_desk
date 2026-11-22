#Requires -Version 7
# Brands a checkout as PowerPCU Remote before a build. Run from anywhere; CI runs it
# before cargo/flutter. The repo itself keeps RustDesk's files, so a build without
# this script (or without the `powerpcu` feature) stays stock.
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$out = Join-Path $PSScriptRoot 'out'
$utf8 = [System.Text.UTF8Encoding]::new($false)

$edits = @(
    @{
        File  = 'flutter/windows/runner/Runner.rc'
        Pairs = @(
            , @('"CompanyName", "Purslane Tech Pte. Ltd."', '"CompanyName", "WorkDii"')
            , @('"FileDescription", "RustDesk Remote Desktop"', '"FileDescription", "PowerPCU Remote"')
            , @('"InternalName", "rustdesk"', '"InternalName", "powerpcu-remote"')
            , @('All rights reserved."', 'All rights reserved. Modified by WorkDii (AGPL-3.0)."')
            , @('"OriginalFilename", "rustdesk.exe"', '"OriginalFilename", "powerpcu-remote.exe"')
            , @('"ProductName", "RustDesk"', '"ProductName", "PowerPCU Remote"')
        )
    },
    @{
        File  = 'libs/portable/Cargo.toml'
        Pairs = @(
            , @('ProductName = "RustDesk"', 'ProductName = "PowerPCU Remote"')
            , @('OriginalFilename = "rustdesk.exe"', 'OriginalFilename = "powerpcu-remote-x86_64.exe"')
            , @('FileDescription = "RustDesk Remote Desktop"', 'FileDescription = "PowerPCU Remote"')
        )
    }
)

$copies = [ordered]@{
    'app_icon.ico'    = 'flutter/windows/runner/resources/app_icon.ico'
    'icon.ico'        = 'res/icon.ico'
    'tray-icon.ico'   = 'res/tray-icon.ico'
    'assets-icon.ico' = 'flutter/assets/icon.ico'
    'assets-icon.png' = 'flutter/assets/icon.png'
    'icon.svg'        = 'flutter/assets/icon.svg'
    'logo.png'        = 'flutter/assets/logo.png'
    'logo_dark.png'   = 'flutter/assets/logo_dark.png'
}

# Every anchor is checked before anything is written, so a moved upstream line or a
# second run fails without leaving a half-branded tree.
$pending = foreach ($edit in $edits) {
    $path = Join-Path $root $edit.File
    $text = [System.IO.File]::ReadAllText($path, $utf8)
    foreach ($pair in $edit.Pairs) {
        if (-not $text.Contains($pair[0])) {
            throw "apply-branding: '$($pair[0])' not found in $($edit.File) (already branded, or upstream changed it)"
        }
        $text = $text.Replace($pair[0], $pair[1])
    }
    [pscustomobject]@{ Path = $path; Text = $text }
}
foreach ($name in $copies.Keys) {
    if (-not (Test-Path -LiteralPath (Join-Path $out $name) -PathType Leaf)) {
        throw "apply-branding: missing res/powerpcu/out/$name (run gen-icons.mjs)"
    }
}

foreach ($name in $copies.Keys) {
    Copy-Item -LiteralPath (Join-Path $out $name) -Destination (Join-Path $root $copies[$name]) -Force
}
foreach ($file in $pending) {
    [System.IO.File]::WriteAllText($file.Path, $file.Text, $utf8)
}
Write-Host "apply-branding: branded $($pending.Count) files, copied $($copies.Count) assets"
