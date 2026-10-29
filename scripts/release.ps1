# Cuts a release that installed copies pick up on their own:
#   .\scripts\release.ps1 0.1.2 "Apa yang baru"
# Bumps the version, builds the signed installer, commits, tags and publishes a
# GitHub release with the installer, its signature, latest.json and a portable zip.
# Needs the signing key at %USERPROFILE%\.tauri\notchy.key (never commit it).
param(
    [Parameter(Mandatory)][string]$Version,
    [string]$Notes = ""
)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot
Set-Location $root

if (git status --porcelain) { throw "Commit or stash your changes first." }

# 1. Version, in all three places.
$files = @{
    "package.json"              = '("version":\s*")[^"]+(")'
    "src-tauri/tauri.conf.json" = '("version":\s*")[^"]+(")'
    "src-tauri/Cargo.toml"      = '(?m)^(version = ")[^"]+(")'
}
foreach ($f in $files.Keys) {
    $text = [IO.File]::ReadAllText("$root/$f")
    [IO.File]::WriteAllText("$root/$f", ([regex]$files[$f]).Replace($text, "`${1}$Version`${2}", 1))
}

# 2. Build. The watchdog would restart Notchy mid-build and lock the exe, so pause it.
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "$env:USERPROFILE\.tauri\notchy.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
$watchdog = Get-ScheduledTask "Notchy Watchdog" -ErrorAction SilentlyContinue
if ($watchdog) { Disable-ScheduledTask "Notchy Watchdog" | Out-Null }
Stop-Process -Name notchy -ErrorAction SilentlyContinue
try {
    npm run tauri build -- --no-bundle
    if ($LASTEXITCODE) { throw "Build failed." }
    # Portable: the plain exe, zipped before packing, because packing marks this
    # very file as installed (and the installed kind would try to run an installer).
    # The antivirus scans the fresh exe and holds it meanwhile, hence the retries.
    $exe = "$root\src-tauri\target\release\notchy.exe"
    if ([IO.File]::ReadAllText($exe, [Text.Encoding]::ASCII).Contains("__TAURI_BUNDLE_TYPE_VAR_NSS")) { throw "notchy.exe is marked as installed; not fit for the portable zip." }
    $portable = "$root\src-tauri\target\release\portable\Notchy_${Version}_x64-portable.zip"
    New-Item -ItemType Directory -Force (Split-Path $portable) | Out-Null
    foreach ($try in 1..5) {
        Start-Sleep -Seconds 20
        try { Compress-Archive -Path $exe -DestinationPath $portable -Force; break }
        catch { if ($try -eq 5) { throw } }
    }
    # Same antivirus hold (os error 1224 when packing), so retry the packing alone.
    foreach ($try in 1..5) {
        Start-Sleep -Seconds 20
        npx tauri bundle --bundles nsis
        if (-not $LASTEXITCODE) { break }
        if ($try -eq 5) { throw "Packing the installer failed." }
    }
} finally {
    Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = "`"$root\src-tauri\target\release\notchy.exe`"" } | Out-Null
    if ($watchdog) { Enable-ScheduledTask "Notchy Watchdog" | Out-Null }
}

# 3. latest.json: what installed copies check.
$name = "Notchy_${Version}_x64-setup.exe"
$setup = "$root\src-tauri\target\release\bundle\nsis\$name"
$latest = "$root\src-tauri\target\release\bundle\nsis\latest.json"
$json = @{
    version   = $Version
    notes     = $Notes
    pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = (Get-Content "$setup.sig" -Raw).Trim()
            url       = "https://github.com/RamantyaDS/Notchy/releases/download/v$Version/$name"
        }
    }
} | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText($latest, $json)  # no BOM
# 4. Publish.
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "Release $Version"
git tag "v$Version"
git push origin HEAD "v$Version"
# The fingerprints go in the release notes, so anyone can check their download.
$sha = (Get-FileHash $setup -Algorithm SHA256).Hash.ToLower()
$zipSha = (Get-FileHash $portable -Algorithm SHA256).Hash.ToLower()
$zipName = Split-Path $portable -Leaf
$page = "$Notes`n`n**Portable** (tanpa instal, tanpa update otomatis): ``$zipName```n`n**SHA-256**`n- ``$name``: ``$sha```n- ``$zipName``: ``$zipSha``"
gh release create "v$Version" $setup "$setup.sig" $latest $portable --title "Notchy $Version" --notes $page
Write-Host "Released v$Version. Installed copies (v0.2.1+) pick it up within an hour, after waking from sleep, or on their next start."
