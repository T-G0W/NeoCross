# Builds the Windows release into dist\release: the setup (x64 + x86 in one) and a portable zip.
# Needs Rust (MSVC) with both targets and Inno Setup 6 (winget install JRSoftware.InnoSetup).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$version = [regex]::Match((Get-Content Cargo.toml -Raw), '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if ($env:GITHUB_REF_NAME -and $env:GITHUB_REF_NAME -ne "v$version") {
    throw "Tag $env:GITHUB_REF_NAME does not match version $version in Cargo.toml"
}
Write-Host "== NeoCross $version" -ForegroundColor Cyan

# No local paths in the executables.
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $HOME ".cargo" }
$sep = [char]0x1f
$env:CARGO_ENCODED_RUSTFLAGS = "--remap-path-prefix=$cargoHome=/cargo$sep--remap-path-prefix=$root=/neocross"
$targets = @("x86_64-pc-windows-msvc", "i686-pc-windows-msvc")
try {
    foreach ($t in $targets) {
        Write-Host "== Build $t" -ForegroundColor Cyan
        rustup target add $t | Out-Null
        cargo build --release --locked --target $t
        if ($LASTEXITCODE -ne 0) { throw "Build failed ($t)" }
    }
} finally {
    Remove-Item Env:CARGO_ENCODED_RUSTFLAGS -ErrorAction SilentlyContinue
}
foreach ($t in $targets) {
    $bytes = [IO.File]::ReadAllBytes("target\$t\release\neocross.exe")
    $text = [Text.Encoding]::ASCII.GetString($bytes) + [Text.Encoding]::Unicode.GetString($bytes)
    foreach ($bad in @(":\Users\", $root)) {
        if ($text.Contains($bad)) { throw "Local path ($bad) in target\$t\release\neocross.exe" }
    }
}
Write-Host "No local paths in the executables."

cargo run --release --locked --manifest-path tools\licenses\Cargo.toml -- --target x86_64-pc-windows-msvc --out target\THIRD_PARTY_LICENSES.txt
if ($LASTEXITCODE -ne 0) { throw "License collection failed" }

$out = Join-Path $root "dist\release"
New-Item -ItemType Directory -Force $out | Out-Null
Get-ChildItem $out -File -Filter "NeoCross-*" | Where-Object { $_.Name -match "Setup|windows" } | Remove-Item

$iscc = @(
    (Join-Path $env:LOCALAPPDATA "Programs\Inno Setup 6\ISCC.exe"),
    (Join-Path ${env:ProgramFiles(x86)} "Inno Setup 6\ISCC.exe"),
    (Join-Path $env:ProgramFiles "Inno Setup 6\ISCC.exe")
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) { throw "Inno Setup 6 not found (winget install JRSoftware.InnoSetup)" }
& $iscc /Qp "/DAppVersion=$version" installer\neocross.iss
if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed" }

# Portable: one exe, settings in %APPDATA%\T-GOW\NeoCross (or NEOCROSS_DATA_DIR).
$stage = Join-Path $root "dist\portable"
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item target\x86_64-pc-windows-msvc\release\neocross.exe (Join-Path $stage "NeoCross.exe")
Copy-Item LICENSE-MIT (Join-Path $stage "LICENSE-MIT.txt")
Copy-Item LICENSE-APACHE (Join-Path $stage "LICENSE-APACHE.txt")
Copy-Item target\THIRD_PARTY_LICENSES.txt $stage
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath (Join-Path $out "NeoCross-$version-windows-x64-portable.zip") -Force

$notes = foreach ($lang in @("en", "de")) {
    $f = "release\notes-$lang.txt"
    if (Test-Path $f) { (Get-Content $f -Raw -Encoding UTF8).Trim() }
}
$body = "## NeoCross $version`n`n" + (@($notes | Where-Object { $_ }) -join "`n`n---`n`n") + "`n"
[IO.File]::WriteAllText((Join-Path $out "RELEASE_NOTES.md"), $body, (New-Object Text.UTF8Encoding $false))

Write-Host "== Done: $out" -ForegroundColor Green
Get-ChildItem $out -File | Format-Table Name, @{ n = "MB"; e = { [math]::Round($_.Length / 1MB, 2) } } -AutoSize
