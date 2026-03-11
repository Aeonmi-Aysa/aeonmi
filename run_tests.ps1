# Test script for Aeonmi
Set-Location "C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01"
Write-Host "Running cargo test..." -ForegroundColor Cyan
cargo test --no-fail-fast 2>&1 | Tee-Object -FilePath test_output.txt
Write-Host "`nTest complete. Results saved to test_output.txt" -ForegroundColor Green
Write-Host "`nPassing tests:" -ForegroundColor Green
Select-String -Path test_output.txt -Pattern "test .* \.\.\. ok" | Select-Object -First 20
Write-Host "`nFailing tests:" -ForegroundColor Red
Select-String -Path test_output.txt -Pattern "test .* \.\.\. FAILED" | Select-Object -First 20
