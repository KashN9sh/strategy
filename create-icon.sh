#!/bin/bash
# Скрипт для создания иконки приложения Cozy Kingdom

set -e

ICONSET_DIR="Cozy Kingdom.iconset"
ICNS_FILE="Cozy Kingdom.icns"
SOURCE_IMAGE=""

# Цвета
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🎨 Создание иконки для Cozy Kingdom"
echo ""

# Проверка наличия исходного изображения
if [ -n "$1" ]; then
    SOURCE_IMAGE="$1"
elif [ -f "assets/icon.png" ]; then
    SOURCE_IMAGE="assets/icon.png"
    echo -e "${YELLOW}Используется assets/icon.png как источник${NC}"
    echo "Можно указать другой файл: ./create-icon.sh path/to/image.png"
    echo ""
else
    echo "❌ Не найден исходный файл изображения"
    echo ""
    echo "Использование:"
    echo "  ./create-icon.sh [путь_к_изображению.png]"
    echo ""
    echo "Или создайте PNG файл 1024x1024 пикселей и назовите его icon.png"
    exit 1
fi

if [ ! -f "$SOURCE_IMAGE" ]; then
    echo "❌ Файл не найден: $SOURCE_IMAGE"
    exit 1
fi

# Создаем временную директорию для iconset
rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

echo "📐 Генерирую размеры иконки..."

# macOS требует следующие размеры для .icns:
# icon_16x16.png
# icon_16x16@2x.png (32x32)
# icon_32x32.png
# icon_32x32@2x.png (64x64)
# icon_128x128.png
# icon_128x128@2x.png (256x256)
# icon_256x256.png
# icon_256x256@2x.png (512x512)
# icon_512x512.png
# icon_512x512@2x.png (1024x1024)

sips -z 16 16 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_16x16.png" > /dev/null
sips -z 32 32 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_16x16@2x.png" > /dev/null
sips -z 32 32 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_32x32.png" > /dev/null
sips -z 64 64 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_32x32@2x.png" > /dev/null
sips -z 128 128 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_128x128.png" > /dev/null
sips -z 256 256 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_128x128@2x.png" > /dev/null
sips -z 256 256 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_256x256.png" > /dev/null
sips -z 512 512 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_256x256@2x.png" > /dev/null
sips -z 512 512 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_512x512.png" > /dev/null
sips -z 1024 1024 "$SOURCE_IMAGE" --out "$ICONSET_DIR/icon_512x512@2x.png" > /dev/null

echo "📦 Создаю .icns файл..."

# Конвертируем iconset в icns
iconutil -c icns "$ICONSET_DIR" -o "$ICNS_FILE"

# Удаляем временную директорию
rm -rf "$ICONSET_DIR"

echo -e "${GREEN}✅ Иконка создана: $ICNS_FILE${NC}"
echo ""

# Копируем в bundle если он существует
if [ -d "target/release/bundle/osx/Cozy Kingdom.app" ]; then
    echo "📱 Копирую иконку в bundle..."
    cp "$ICNS_FILE" "target/release/bundle/osx/Cozy Kingdom.app/Contents/Resources/"
    
    # Обновляем Info.plist
    echo "📝 Обновляю Info.plist..."
    PLIST="target/release/bundle/osx/Cozy Kingdom.app/Contents/Info.plist"
    if [ -f "$PLIST" ]; then
        # Используем PlistBuddy для добавления CFBundleIconFile
        /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile Cozy Kingdom" "$PLIST" 2>/dev/null || \
        /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string Cozy Kingdom" "$PLIST" 2>/dev/null
        
        echo -e "${GREEN}✅ Иконка добавлена в bundle${NC}"
    fi
else
    echo "💡 Bundle не найден. Иконка будет добавлена при следующей сборке."
fi

echo ""
echo "✅ Готово! Иконка: $ICNS_FILE"
