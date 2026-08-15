<#
.SYNOPSIS
    Copies the most recent day_N folder to day_(N+1) and commits it.

.DESCRIPTION
    Scans the repo root for day_<number> directories, finds the highest
    numbered one, copies its contents -- including the "target" build
    cache, so dependencies like bevy don't need to recompile -- into a
    new day_<number+1> directory, stages the source (target/ stays
    gitignored either way), and creates a commit "day <number+1>".

    Afterwards, the "target" directory is deleted from every OTHER
    day_N folder to reclaim disk space, since only the newest day is
    still being actively built.

.PARAMETER Message
    Optional custom commit message. Defaults to "day <number>".

.PARAMETER NoCommit
    Skip staging/committing; just create the new day directory.

.PARAMETER KeepOldTargets
    Skip deleting target/ from older day_N folders.
#>
[CmdletBinding()]
param(
    [string]$Message,
    [switch]$NoCommit,
    [switch]$KeepOldTargets
)

$ErrorActionPreference = "Stop"

$repoRoot = git rev-parse --show-toplevel 2>$null
if (-not $repoRoot) {
    throw "Not inside a git repository."
}
Set-Location $repoRoot

$dayDirs = Get-ChildItem -Directory -Path $repoRoot |
    Where-Object { $_.Name -match '^day_(\d+)$' } |
    ForEach-Object {
        [PSCustomObject]@{
            Number = [int]$Matches[1]
            Path   = $_.FullName
            Name   = $_.Name
        }
    } |
    Sort-Object Number

if (-not $dayDirs) {
    throw "No day_<N> directories found in $repoRoot."
}

$latest = $dayDirs[-1]
$nextNumber = $latest.Number + 1
$nextName = "day_$nextNumber"
$nextPath = Join-Path $repoRoot $nextName

if (Test-Path $nextPath) {
    throw "$nextName already exists."
}

Write-Host "Copying $($latest.Name) -> $nextName (including target/, this may take a bit) ..."

# /MT multithreads the copy, which helps a lot with the huge number of
# small files in a Rust "target" incremental-build directory.
robocopy $latest.Path $nextPath /E /MT:16 /NFL /NDL /NJH /NJS | Out-Null
$robocopyExit = $LASTEXITCODE
if ($robocopyExit -ge 8) {
    throw "robocopy failed with exit code $robocopyExit"
}

Write-Host "Created $nextName."

# disabled for now as rust-analyzer and other tools are now caching in target/ across days, so we don't want to delete it yet
#if (-not $KeepOldTargets) {
#    foreach ($day in $dayDirs) {
#        $targetPath = Join-Path $day.Path "target"
#        if (Test-Path $targetPath) {
#            Write-Host "Removing $($day.Name)\target ..."
#            Remove-Item -Path $targetPath -Recurse -Force
#        }
#    }
#}

if ($NoCommit) {
    Write-Host "Skipping commit (-NoCommit passed)."
    exit 0
}

git add -- $nextName
if (-not $Message) {
    $Message = "day $nextNumber"
}
git commit -m $Message

Write-Host "Committed '$Message'."
exit 0
