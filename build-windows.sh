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

# Создание портативной версии для NSIS
echo ""
echo "📦 Creating portable package for NSIS installer..."
PACKAGE_DIR="Cozy Kingdom Portable"
rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

# Копируем exe
if [ -f "target/$TARGET/release/strategy.exe" ]; then
    cp "target/$TARGET/release/strategy.exe" "$PACKAGE_DIR/Cozy Kingdom.exe"
    echo "✅ Copied executable"
else
    echo "❌ Executable not found: target/$TARGET/release/strategy.exe"
    exit 1
fi

# Копируем ресурсы
if [ -d "assets" ]; then
    cp -r assets "$PACKAGE_DIR/"
    echo "✅ Copied assets"
fi

if [ -d "shaders" ]; then
    cp -r shaders "$PACKAGE_DIR/"
    echo "✅ Copied shaders"
fi

# Копируем LICENSE для NSIS
if [ -f "LICENSE" ]; then
    cp LICENSE "$PACKAGE_DIR/"
    echo "✅ Copied LICENSE"
fi

# Создаем ZIP
echo "📦 Creating ZIP archive..."
rm -f "Cozy Kingdom Portable.zip"
zip -r "Cozy Kingdom Portable.zip" "$PACKAGE_DIR" > /dev/null

echo ""
echo "✅ Portable package created: Cozy Kingdom Portable.zip"
echo ""
echo "💡 Для создания NSIS установщика:"
echo "   1. Установите NSIS на Windows: https://nsis.sourceforge.io/Download"
echo "   2. Скопируйте папку '$PACKAGE_DIR' на Windows"
echo "   3. Скопируйте installer.nsi на Windows"
echo "   4. Запустите: makensis installer.nsi"
echo ""
echo "   Или используйте GitHub Actions для автоматической сборки!"
