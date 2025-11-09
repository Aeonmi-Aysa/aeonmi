# Test script for crash diagnostics
# This script will help identify exactly where the executable is crashing

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
    Write-Host "✓ Version check passed" -ForegroundColor Green
} catch {
    Write-Host "✗ Version check failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 2: Help display
Write-Host "Test 2: Help display..." -ForegroundColor Cyan
try {
    & ".\target\release\aeonmi_shard.exe" --help
    Write-Host "✓ Help display passed" -ForegroundColor Green
} catch {
    Write-Host "✗ Help display failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Just running the executable with no args
Write-Host "Test 3: Running with no arguments..." -ForegroundColor Cyan
try {
    & ".\target\release\aeonmi_shard.exe"
    Write-Host "✓ No arguments test passed" -ForegroundColor Green
} catch {
    Write-Host "✗ No arguments test failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 4: Invalid command to test error handling
Write-Host "Test 4: Invalid command test..." -ForegroundColor Cyan
try {
    $result = & ".\target\release\aeonmi_shard.exe" invalid_command 2>&1
    Write-Host "✓ Invalid command handled gracefully" -ForegroundColor Green
} catch {
    Write-Host "✗ Invalid command caused crash: $_" -ForegroundColor Red
}
Write-Host ""

# Test 5: Running a simple test file
Write-Host "Test 5: Running simple test file..." -ForegroundColor Cyan
if (Test-Path "simple_test.aeon") {
    try {
        & ".\target\release\aeonmi_shard.exe" run simple_test.aeon
        Write-Host "✓ Simple test file execution passed" -ForegroundColor Green
    } catch {
        Write-Host "✗ Simple test file execution failed: $_" -ForegroundColor Red
    }
} else {
    Write-Host "⚠ simple_test.aeon not found, skipping" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "=== Diagnostic Tests Complete ===" -ForegroundColor Green
Write-Host "If any tests failed above, that indicates where the crash is occurring." -ForegroundColor Yellow
Write-Host "Please run this script and share the output to help identify the issue." -ForegroundColor Yellow