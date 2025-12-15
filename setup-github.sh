#!/bin/bash
# GLifzip GitHub Repository Setup Script
# This script creates the GitHub repository and pushes the initial release

set -e

echo "=========================================="
echo "GLifzip GitHub Repository Setup"
echo "=========================================="
echo ""

# Step 1: Verify we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Must run from glifzip directory"
    exit 1
fi

echo "[1/6] Verifying git repository..."
if [ ! -d ".git" ]; then
    echo "Error: Not a git repository"
    exit 1
fi
echo "✓ Git repository verified"
echo ""

# Step 2: Check GitHub CLI authentication
echo "[2/6] Checking GitHub CLI authentication..."
if ! gh auth status &>/dev/null; then
    echo "GitHub CLI not authenticated. Please run:"
    echo "  gh auth login"
    echo ""
    echo "Choose:"
    echo "  - GitHub.com"
    echo "  - HTTPS"
    echo "  - Authenticate with web browser"
    exit 1
fi
echo "✓ GitHub CLI authenticated"
echo ""

# Step 3: Create GitHub repository
echo "[3/6] Creating GitHub repository EarthwebAP/glifzip..."
if gh repo view EarthwebAP/glifzip &>/dev/null; then
    echo "⚠ Repository already exists, skipping creation"
else
    gh repo create EarthwebAP/glifzip \
        --public \
        --source=. \
        --description="High-performance compression engine for GlyphOS - 10-100× faster than traditional ZIP" \
        --remote=origin
    echo "✓ Repository created"
fi
echo ""

# Step 4: Push code to GitHub
echo "[4/6] Pushing code to GitHub..."
git push -u origin main 2>/dev/null || git push -u origin master
echo "✓ Code pushed"
echo ""

# Step 5: Push tags to GitHub
echo "[5/6] Pushing tags to GitHub..."
git push --tags
echo "✓ Tags pushed"
echo ""

# Step 6: Build release binary
echo "[6/6] Building release binary..."
cargo build --release
echo "✓ Release binary built"
echo ""

echo "=========================================="
echo "GitHub Repository Setup Complete!"
echo "=========================================="
echo ""
echo "Repository URL: https://github.com/EarthwebAP/glifzip"
echo ""
echo "Next steps:"
echo "1. Visit https://github.com/EarthwebAP/glifzip/releases"
echo "2. Create a new release from tag v1.0.0"
echo "3. Upload the binary from: target/release/glifzip"
echo ""
echo "Or use the automated release script:"
echo "  ./create-github-release.sh"
echo ""
