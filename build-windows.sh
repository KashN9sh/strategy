#!/bin/bash
# Скрипт для сборки Windows версии Cozy Kingdom
# Запускается через WSL или Git Bash на Windows

set -e

echo "🔨 Building Cozy Kingdom for Windows..."

TARGET="x86_64-pc-windows-msvc"

# Проверяем наличие target
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "⚠️  Windows target not installed. Installing..."
    rustup target add $TARGET
fi

# Сборка release версии
echo "📦 Building release..."
cargo build --release --target $TARGET

# Создание .msi установщика
echo "📦 Creating .msi installer..."
cargo bundle --release --target $TARGET

echo "✅ Build complete!"
echo "📍 Installer location: target/release/bundle/msi/"

# Альтернатива: создание портативной версии
echo ""
read -p "Create portable ZIP package? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📦 Creating portable package..."
    PACKAGE_DIR="Cozy Kingdom Portable"
    mkdir -p "$PACKAGE_DIR"
    
    # Копируем exe
    cp "target/release/$TARGET/strategy.exe" "$PACKAGE_DIR/Cozy Kingdom.exe"
    
    # Копируем ресурсы
    cp -r assets "$PACKAGE_DIR/"
    cp -r shaders "$PACKAGE_DIR/"
    
    # Создаем ZIP
    zip -r "Cozy Kingdom Portable.zip" "$PACKAGE_DIR"
    rm -rf "$PACKAGE_DIR"
    
    echo "✅ Portable package created: Cozy Kingdom Portable.zip"
fi
