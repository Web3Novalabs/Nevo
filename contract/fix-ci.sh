#!/bin/bash

# Script to fix common CI issues for the contract

set -e

echo "🔧 Fixing CI Issues for Contract..."
echo ""

# Navigate to contract directory
cd "$(dirname "$0")/contract"

echo "1️⃣ Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi
echo "✅ Cargo found: $(cargo --version)"
echo ""

echo "2️⃣ Checking code formatting..."
if cargo fmt --all -- --check; then
    echo "✅ Code formatting is correct"
else
    echo "⚠️  Formatting issues found. Running cargo fmt..."
    cargo fmt --all
    echo "✅ Code formatted"
fi
echo ""

echo "3️⃣ Running clippy checks..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✅ No clippy warnings"
else
    echo "⚠️  Clippy warnings found. Please review and fix."
fi
echo ""

echo "4️⃣ Running tests..."
if cargo test; then
    echo "✅ All tests passed"
else
    echo "❌ Some tests failed. Please review."
    exit 1
fi
echo ""

echo "5️⃣ Building release..."
if cargo build --release; then
    echo "✅ Release build successful"
else
    echo "❌ Release build failed"
    exit 1
fi
echo ""

echo "🎉 All CI checks passed!"
echo ""
echo "Next steps:"
echo "  - Review any warnings above"
echo "  - Commit your changes: git add . && git commit -m 'Fix CI issues'"
echo "  - Push to trigger CI: git push"
