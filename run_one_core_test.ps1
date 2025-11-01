<#
.SYNOPSIS
    Builds and runs the Steady "standard" example in Docker, restricted to 1 CPU core.

.DESCRIPTION
    This script builds the Docker image using the Dockerfile located in the current folder
    (steady-state-standard), but uses the parent directory as the Docker build context
    so Docker can access the sibling crate steady-state-stack/core.

.REQUIREMENTS
    - PowerShell 5.0 or later
    - Docker Desktop or Docker Engine installed and running
#>

param(
    [string]$ImageName = "steady-standard",
    [string]$Tag = "latest"
)

# Treat all errors as terminating
$ErrorActionPreference = "Stop"

# Compute the absolute path to the Dockerfile and parent context
$ProjectDir = (Get-Location).Path
$DockerfilePath = Join-Path $ProjectDir "Dockerfile"
$ContextPath = Split-Path -Parent $ProjectDir

Write-Host "Building container image ${ImageName}:${Tag} ..." -ForegroundColor Cyan
Write-Host "Dockerfile: $DockerfilePath"
Write-Host "Context:    $ContextPath"

# Build image with correct paths
docker build `
    -t ("${ImageName}:${Tag}") `
    -f "$DockerfilePath" `
    "$ContextPath"

if ($LASTEXITCODE -ne 0) {
    Write-Error "Docker build failed. Fix any errors above and rerun."
    exit 1
}

Write-Host ""
Write-Host "Running container with 1 simulated CPU core..." -ForegroundColor Green

# Run the container with a single CPU core available and expose telemetry port
docker run `
    --rm `
    -it `
    --cpus=1 `
    -p 9900:9900 `
    ("${ImageName}:${Tag}")

if ($LASTEXITCODE -eq 0) {
    Write-Host "Run completed successfully." -ForegroundColor Green
} else {
    Write-Warning ("Container exited with non-zero code ({0}). Check logs above." -f $LASTEXITCODE)
}

Write-Host ""
Write-Host "Verification steps:" -ForegroundColor Yellow
Write-Host "  1. Look for 'Spawning SoloAct ... on new OS thread' in log output."
Write-Host "  2. Open http://127.0.0.1:9900 to verify telemetry dashboard appears."
Write-Host "  3. Ensure system terminates cleanly after beats complete."