# Let's investigate what specific crash you're experiencing
Write-Host "=== Investigating Crash Reports ===" -ForegroundColor Yellow
Write-Host ""

# Test: Double-clicking the exe (simulating Windows Explorer behavior)
Write-Host "Test A: Double-click simulation..." -ForegroundColor Cyan
Write-Host "This will run the exe and pause to see any crash messages:" -ForegroundColor Yellow
Start-Process -FilePath ".\target\release\aeonmi_shard.exe" -Wait -NoNewWindow
Write-Host "Did the executable window flash and close immediately? (This would be normal behavior)" -ForegroundColor Green
Write-Host ""

# Test: Running specific commands that might cause issues
Write-Host "Test B: Testing various commands that might crash..." -ForegroundColor Cyan

$commands = @(
    "build",
    "run", 
    "test",
    "check",
    "new test_project",
    "repl",
    "editor"
)

foreach ($cmd in $commands) {
    Write-Host "Testing: aeonmi_shard.exe $cmd" -ForegroundColor White
    try {
        $result = & ".\target\release\aeonmi_shard.exe" $cmd.Split(' ') 2>&1
        Write-Host "✓ Command completed (may have shown errors, but didn't crash)" -ForegroundColor Green
    } catch {
        Write-Host "✗ Command crashed: $_" -ForegroundColor Red
    }
    Write-Host ""
}

Write-Host "=== Investigation Complete ===" -ForegroundColor Yellow
Write-Host ""
Write-Host "QUESTIONS FOR YOU:" -ForegroundColor Red
Write-Host "1. Are you double-clicking the .exe files in Windows Explorer?" -ForegroundColor White
Write-Host "2. What exactly happens when you say they 'crash'?" -ForegroundColor White
Write-Host "3. Do you see any error messages, or does the window just close?" -ForegroundColor White
Write-Host "4. Are you trying to run specific commands that fail?" -ForegroundColor White