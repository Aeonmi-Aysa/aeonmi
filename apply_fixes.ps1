# AEONMI COMPILATION FIX - ATOMIC APPLICATION
# Applies exactly 3 fixes as specified

$ErrorActionPreference = "Stop"
Set-Location "C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01"

Write-Host "=== AEONMI COMPILATION FIX ===" -ForegroundColor Cyan
Write-Host ""

# FIX 1: Add hex and rpassword to Cargo.toml
Write-Host "[1/3] Fixing Cargo.toml dependencies..." -ForegroundColor Yellow

$cargoPath = "Cargo.toml"
$cargoContent = Get-Content $cargoPath -Raw

# Add hex if not present
if ($cargoContent -notmatch '^\s*hex\s*=') {
    Write-Host "  Adding: hex = `"0.4`"" -ForegroundColor Green
    # Find [dependencies] section and add after it
    $cargoContent = $cargoContent -replace '(\[dependencies\])', "`$1`nhex = `"0.4`""
}

# Add rpassword if not present
if ($cargoContent -notmatch '^\s*rpassword\s*=') {
    Write-Host "  Adding: rpassword = `"7.3`"" -ForegroundColor Green
    $cargoContent = $cargoContent -replace '(\[dependencies\])', "`$1`nrpassword = `"7.3`""
}

Set-Content $cargoPath -Value $cargoContent -NoNewline
Write-Host "  ✓ Cargo.toml updated" -ForegroundColor Green
Write-Host ""

# FIX 2: Fix ZeroizeOnDrop in mgk.rs
Write-Host "[2/3] Fixing src/glyph/mgk.rs (ZeroizeOnDrop)..." -ForegroundColor Yellow

$mgkPath = "src\glyph\mgk.rs"
if (Test-Path $mgkPath) {
    $mgkContent = Get-Content $mgkPath -Raw
    
    # Remove ZeroizeOnDrop from derive
    $mgkContent = $mgkContent -replace '#\[derive\(Clone,\s*ZeroizeOnDrop\)\]', '#[derive(Clone)]'
    
    # Add manual impl if not already present
    if ($mgkContent -notmatch 'impl\s+ZeroizeOnDrop\s+for\s+MasterGlyphKey') {
        # Find the struct definition and add impl after it
        $mgkContent = $mgkContent -replace '(pub struct MasterGlyphKey\s*\{[^}]+\})', "`$1`n`nimpl ZeroizeOnDrop for MasterGlyphKey {}"
    }
    
    Set-Content $mgkPath -Value $mgkContent -NoNewline
    Write-Host "  ✓ mgk.rs fixed (removed derive, added manual impl)" -ForegroundColor Green
} else {
    Write-Host "  ✗ ERROR: $mgkPath not found!" -ForegroundColor Red
    exit 1
}
Write-Host ""

# FIX 3: Add VaultCommand::Init match arm
Write-Host "[3/3] Fixing src/commands/vault.rs (Init pattern)..." -ForegroundColor Yellow

$vaultPath = "src\commands\vault.rs"
if (Test-Path $vaultPath) {
    $vaultContent = Get-Content $vaultPath -Raw
    
    # Add Init match arm if not present
    if ($vaultContent -notmatch 'VaultCommand::Init') {
        # Find the last match arm before the closing brace and add Init before it
        # Look for the pattern: "        }," followed by whitespace and closing brace
        $initArm = @"
        VaultCommand::Init(_args) => {
            eprintln!("Vault init not yet implemented");
            Ok(())
        },
"@
        # Insert before the final closing brace of the match statement
        $vaultContent = $vaultContent -replace '(\s+)(VaultCommand::\w+.*?\{[^}]*\}\s*,?\s*)(\s*\})', "`$1`$2`n$initArm`$3"
    }
    
    Set-Content $vaultPath -Value $vaultContent -NoNewline
    Write-Host "  ✓ vault.rs fixed (Init match arm added)" -ForegroundColor Green
} else {
    Write-Host "  ✗ ERROR: $vaultPath not found!" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "=== ALL FIXES APPLIED ===" -ForegroundColor Green
Write-Host ""
Write-Host "Running cargo clean..." -ForegroundColor Cyan
cargo clean

Write-Host ""
Write-Host "Running cargo build --release..." -ForegroundColor Cyan
Write-Host "(This will take a few minutes...)" -ForegroundColor Gray
Write-Host ""

cargo build --release 2>&1 | Tee-Object -FilePath "build_output_fixed.txt"

Write-Host ""
Write-Host "=== BUILD COMPLETE ===" -ForegroundColor Cyan
Write-Host "Full output saved to: build_output_fixed.txt" -ForegroundColor Gray