# Clean build script for Aeonmi
Set-Location "C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01"
Write-Host "Cleaning build cache..." -ForegroundColor Cyan
cargo clean
Write-Host "`nRebuilding project..." -ForegroundColor Cyan
cargo build --release 2>&1 | Tee-Object -FilePath build_output.txt
Write-Host "`nBuild complete. Check build_output.txt for details." -ForegroundColor Green
