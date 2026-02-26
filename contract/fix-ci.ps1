# PowerShell script to fix common CI issues for the contract

Write-Host "🔧 Fixing CI Issues for Contract..." -ForegroundColor Cyan
Write-Host ""

# Navigate to contract directory
Set-Location "$PSScriptRoot\contract"

Write-Host "1️⃣ Checking Rust installation..." -ForegroundColor Yellow
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoPath) {
    Write-Host "❌ Cargo not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    
    Write-Host ""
    Write-Host "Quick install options:" -ForegroundColor Cyan
    Write-Host "  1. Download from: https://rustup.rs/"
    Write-Host "  2. Or run: winget install Rustlang.Rustup"
    Write-Host "  3. Or use: choco install rust"
    Write-Host ""
    Write-Host "After installation, restart your terminal and run this script again."
    exit 1
}
Write-Host "✅ Cargo found: $(cargo --version)" -ForegroundColor Green
Write-Host ""

Write-Host "2️⃣ Checking code formatting..." -ForegroundColor Yellow
$formatCheck = cargo fmt --all -- --check 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Code formatting is correct" -ForegroundColor Green
} else {
    Write-Host "⚠️  Formatting issues found. Running cargo fmt..." -ForegroundColor Yellow
    cargo fmt --all
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Code formatted" -ForegroundColor Green
    } else {
        Write-Host "❌ Formatting failed" -ForegroundColor Red
    }
}
Write-Host ""

Write-Host "3️⃣ Running clippy checks..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ No clippy warnings" -ForegroundColor Green
} else {
    Write-Host "⚠️  Clippy warnings found. Please review and fix." -ForegroundColor Yellow
}
Write-Host ""

Write-Host "4️⃣ Running tests..." -ForegroundColor Yellow
cargo test
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ All tests passed" -ForegroundColor Green
} else {
    Write-Host "❌ Some tests failed. Please review." -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "5️⃣ Building release..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Release build successful" -ForegroundColor Green
} else {
    Write-Host "❌ Release build failed" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "🎉 All CI checks passed!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  - Review any warnings above"
Write-Host "  - Commit your changes: git add . ; git commit -m 'Fix CI issues'"
Write-Host "  - Push to trigger CI: git push"
