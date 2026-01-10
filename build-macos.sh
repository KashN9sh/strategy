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

# Создание .app bundle вручную (так как cargo-bundle может не работать)
echo "📱 Creating .app bundle..."
mkdir -p "target/release/bundle/osx/Cozy Kingdom.app/Contents/MacOS"
mkdir -p "target/release/bundle/osx/Cozy Kingdom.app/Contents/Resources"

# Копируем бинарник
cp "target/$TARGET/release/strategy" "target/release/bundle/osx/Cozy Kingdom.app/Contents/MacOS/Cozy Kingdom"
chmod +x "target/release/bundle/osx/Cozy Kingdom.app/Contents/MacOS/Cozy Kingdom"

# Копируем ресурсы
cp -r assets "target/release/bundle/osx/Cozy Kingdom.app/Contents/Resources/"
cp -r shaders "target/release/bundle/osx/Cozy Kingdom.app/Contents/Resources/"
if [ -f config.toml ]; then
    cp config.toml "target/release/bundle/osx/Cozy Kingdom.app/Contents/Resources/"
fi

# Создаем Info.plist
cat > "target/release/bundle/osx/Cozy Kingdom.app/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>Cozy Kingdom</string>
    <key>CFBundleIconFile</key>
    <string>Cozy Kingdom</string>
    <key>CFBundleIdentifier</key>
    <string>com.yourcompany.cozykingdom</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Cozy Kingdom</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Добавляем иконку если она существует
if [ -f "Cozy Kingdom.icns" ]; then
    echo "🎨 Добавляю иконку..."
    cp "Cozy Kingdom.icns" "target/release/bundle/osx/Cozy Kingdom.app/Contents/Resources/"
fi

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
