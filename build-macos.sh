#!/bin/bash
# Скрипт для сборки macOS версии Cozy Kingdom

set -e

echo "🔨 Building Cozy Kingdom for macOS..."

# Определяем архитектуру
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-apple-darwin"
    echo "📱 Target: Apple Silicon (ARM64)"
else
    TARGET="x86_64-apple-darwin"
    echo "💻 Target: Intel (x86_64)"
fi

# Сборка release версии
echo "📦 Building release..."
cargo build --release --target $TARGET

# Создание .app bundle
echo "📱 Creating .app bundle..."
cargo bundle --release --target $TARGET

echo "✅ Build complete!"
echo "📍 App location: target/release/bundle/osx/Cozy Kingdom.app"

# Проверяем наличие create-dmg для создания DMG
if command -v create-dmg &> /dev/null; then
    echo ""
    read -p "Create DMG installer? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "💿 Creating DMG..."
        create-dmg \
          --volname "Cozy Kingdom" \
          --window-pos 200 120 \
          --window-size 800 400 \
          --icon-size 100 \
          --icon "Cozy Kingdom.app" 200 190 \
          --hide-extension "Cozy Kingdom.app" \
          --app-drop-link 600 185 \
          "Cozy Kingdom.dmg" \
          "target/release/bundle/osx/"
        echo "✅ DMG created: Cozy Kingdom.dmg"
    fi
else
    echo "💡 Tip: Install 'create-dmg' (brew install create-dmg) to create DMG installer"
fi
