# Test script for crash diagnostics
Write-Host "=== Aeonmi Shard Crash Diagnostics ===" -ForegroundColor Green
Write-Host "Running diagnostic tests to identify crash location..." -ForegroundColor Yellow
Write-Host ""

# Set environment variables for better error reporting
$env:RUST_BACKTRACE = "full"
$env:RUST_LOG = "debug"

# Test 1: Version check
Write-Host "Test 1: Version check..." -ForegroundColor Cyan
try {
    & ".\target\release\aeonmi_shard.exe" --version
    Write-Host "Version check passed" -ForegroundColor Green
} catch {
    Write-Host "Version check failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 2: Help display
Write-Host "Test 2: Help display..." -ForegroundColor Cyan
try {
    & ".\target\release\aeonmi_shard.exe" --help
    Write-Host "Help display passed" -ForegroundColor Green
} catch {
    Write-Host "Help display failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Just running the executable with no args
Write-Host "Test 3: Running with no arguments..." -ForegroundColor Cyan
try {
    & ".\target\release\aeonmi_shard.exe"
    Write-Host "No arguments test passed" -ForegroundColor Green
} catch {
    Write-Host "No arguments test failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== Diagnostic Tests Complete ===" -ForegroundColor Green