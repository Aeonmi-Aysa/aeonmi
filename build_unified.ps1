# AEONMI Unified Build Script
# Creates standalone executable with all ecosystem components
# Windows PowerShell version

param(
    [string]$BuildType = "full-suite",
    [switch]$Release,
    [switch]$Verbose,
    [switch]$Clean
)

Write-Host "AEONMI Unified Build System" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Clean previous builds if requested
if ($Clean) {
    Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
        Write-Host "   Cleaned target directory" -ForegroundColor Green
    }
}

# Available build configurations
$BuildConfigs = @{
    "full-suite" = @{
        description = "Complete AEONMI ecosystem (Mother AI + Titan Libraries + Quantum Vault + Voice + Holographic)"
        features = @("full-suite")
        binary = "aeonmi"
    }
    "mother-ai" = @{
        description = "AEONMI with Mother AI consciousness"
        features = @("mother-ai")
        binary = "aeonmi"
    }
    "titan-only" = @{
        description = "Pure quantum computing with Titan Libraries"
        features = @("titan-libraries")
        binary = "aeonmi"
    }
    "core" = @{
        description = "Core AEONMI Shard compiler only"
        features = @()
        binary = "aeonmi"
    }
    "mother-ai-binary" = @{
        description = "Standalone Mother AI assistant"
        features = @("mother-ai")
        binary = "MotherAI"
    }
}

# Display available configurations
Write-Host "Available build configurations:" -ForegroundColor White
foreach ($config in $BuildConfigs.Keys | Sort-Object) {
    $info = $BuildConfigs[$config]
    Write-Host "   - $config" -ForegroundColor Cyan -NoNewline
    Write-Host " - $($info.description)" -ForegroundColor Gray
}
Write-Host ""

# Validate build type
if (-not $BuildConfigs.ContainsKey($BuildType)) {
    Write-Host "Invalid build type: $BuildType" -ForegroundColor Red
    Write-Host "Available types: $($BuildConfigs.Keys -join ', ')" -ForegroundColor Yellow
    exit 1
}

$config = $BuildConfigs[$BuildType]
Write-Host "Building: $($config.description)" -ForegroundColor Green
Write-Host "Binary: $($config.binary)" -ForegroundColor Green

# Construct cargo command
$cargoArgs = @("build")

if ($Release) {
    $cargoArgs += "--release"
    Write-Host "Release mode enabled" -ForegroundColor Yellow
}

if ($Verbose) {
    $cargoArgs += "--verbose"
}

# Disable default features so each configuration can opt-in explicitly
$cargoArgs += "--no-default-features"

# Add features if specified
if ($config.features.Count -gt 0) {
    $cargoArgs += "--features"
    $cargoArgs += ($config.features -join ",")
    Write-Host "Features: $($config.features -join ', ')" -ForegroundColor Blue
}

# Add binary target if specific binary
if ($config.binary -ne "aeonmi") {
    $cargoArgs += "--bin"
    $cargoArgs += $config.binary
}

Write-Host ""
Write-Host "Building AEONMI..." -ForegroundColor Yellow
Write-Host "Command: cargo $($cargoArgs -join ' ')" -ForegroundColor Gray

# Execute build
$buildStart = Get-Date
try {
    & cargo @cargoArgs
    
    if ($LASTEXITCODE -eq 0) {
        $buildEnd = Get-Date
        $buildTime = ($buildEnd - $buildStart).TotalSeconds
        
    Write-Host ""
    Write-Host "Build completed successfully." -ForegroundColor Green
    Write-Host "Build time: $([math]::Round($buildTime, 2)) seconds" -ForegroundColor Cyan
        
        # Show binary location
        $targetDir = if ($Release) { "target/release" } else { "target/debug" }
        $binaryName = if ($config.binary -eq "aeonmi") { "aeonmi.exe" } else { "$($config.binary).exe" }
        $binaryPath = Join-Path $targetDir $binaryName
        
        if (Test-Path $binaryPath) {
            $binarySize = [math]::Round((Get-Item $binaryPath).Length / 1MB, 2)
            Write-Host "Executable: $binaryPath ($binarySize MB)" -ForegroundColor Cyan
            
            # Show what's included in this build
            Write-Host ""
            Write-Host "Components included:" -ForegroundColor White
            Write-Host "   Shard Quantum Compiler" -ForegroundColor Green
            Write-Host "   AEONMI Shell" -ForegroundColor Green
            
            if ($config.features -contains "full-suite" -or $config.features -contains "mother-ai") {
                Write-Host "   Mother AI Consciousness" -ForegroundColor Green
            }
            
            if ($config.features -contains "full-suite" -or $config.features -contains "titan-libraries") {
                Write-Host "   Titan Quantum Algorithm Libraries" -ForegroundColor Green
                Write-Host "       - Shor's Algorithm (integer factorization)" -ForegroundColor Gray
                Write-Host "       - Grover's Algorithm (quantum search)" -ForegroundColor Gray
                Write-Host "       - Quantum Fourier Transform" -ForegroundColor Gray
                Write-Host "       - Quantum Phase Estimation" -ForegroundColor Gray
                Write-Host "       - Quantum Machine Learning" -ForegroundColor Gray
                Write-Host "       - Quantum Teleportation" -ForegroundColor Gray
            }
            
            if ($config.features -contains "full-suite") {
                Write-Host "   Quantum Vault (secure storage)" -ForegroundColor Green
                Write-Host "   Voice Interface" -ForegroundColor Green
                Write-Host "   Holographic 3D Interface" -ForegroundColor Green
            }
            
            Write-Host ""
            Write-Host "Ready to run:" -ForegroundColor Yellow
            Write-Host "   $binaryPath" -ForegroundColor Cyan
            
            # Usage examples
            Write-Host ""
            Write-Host "Usage examples:" -ForegroundColor White
            if ($config.features -contains "full-suite" -or $config.features -contains "mother-ai") {
                Write-Host "   - Start unified ecosystem: ./$binaryName" -ForegroundColor Cyan
                Write-Host "   - Ask Mother AI: 'Run Shor's algorithm to factor 15'" -ForegroundColor Gray
                Write-Host "   - Quantum search: 'Execute Grover search with 16 items'" -ForegroundColor Gray
            } else {
                Write-Host "   - Start AEONMI shell: ./$binaryName" -ForegroundColor Cyan
                Write-Host "   - Compile quantum program: ./$binaryName compile myprogram.qube" -ForegroundColor Gray
            }
        }
        
    } else {
        Write-Host ""
        Write-Host "Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    
} catch {
    Write-Host ""
    Write-Host "Build failed with error:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "AEONMI unified build finished." -ForegroundColor Magenta