#!/bin/bash
# Скрипт для подписи и нотаризации Cozy Kingdom для macOS

set -e

APP_PATH="target/release/bundle/osx/Cozy Kingdom.app"
DMG_PATH="Cozy Kingdom.dmg"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔐 Подпись приложения Cozy Kingdom"
echo ""

# Проверка наличия приложения
if [ ! -d "$APP_PATH" ]; then
    echo -e "${RED}❌ Приложение не найдено: $APP_PATH${NC}"
    echo "Сначала соберите приложение: ./build-macos.sh"
    exit 1
fi

# Проверка сертификатов
echo "📋 Доступные сертификаты для подписи:"
echo ""
security find-identity -v -p codesigning | grep "Developer ID" || {
    echo -e "${YELLOW}⚠️  Не найдено сертификатов 'Developer ID Application'${NC}"
    echo ""
    echo "Доступные сертификаты:"
    security find-identity -v -p codesigning
    echo ""
}

# Запрос сертификата
if [ -z "$SIGNING_IDENTITY" ]; then
    echo ""
    read -p "Введите имя сертификата для подписи (или нажмите Enter для ad-hoc): " SIGNING_IDENTITY
fi

if [ -z "$SIGNING_IDENTITY" ]; then
    echo -e "${YELLOW}Используется ad-hoc подпись (только для локального тестирования)${NC}"
    SIGNING_IDENTITY="-"
fi

# Подпись приложения
echo ""
echo "✍️  Подписываю приложение..."
codesign --deep --force --verify --verbose \
    --sign "$SIGNING_IDENTITY" \
    --options runtime \
    "$APP_PATH" || {
    echo -e "${RED}❌ Ошибка подписи${NC}"
    exit 1
}

echo ""
echo "✅ Приложение подписано"

# Проверка подписи
echo ""
echo "🔍 Проверяю подпись..."
codesign --verify --verbose "$APP_PATH" || {
    echo -e "${RED}❌ Ошибка проверки подписи${NC}"
    exit 1
}

spctl --assess --verbose "$APP_PATH" && {
    echo -e "${GREEN}✅ Gatekeeper проверка пройдена${NC}"
} || {
    echo -e "${YELLOW}⚠️  Gatekeeper проверка не пройдена (это нормально для ad-hoc подписи)${NC}"
}

# Нотаризация (опционально)
if [ "$SIGNING_IDENTITY" != "-" ] && [ -f "$DMG_PATH" ]; then
    echo ""
    read -p "Нотаризовать DMG? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ -z "$NOTARYTOOL_PROFILE" ]; then
            read -p "Введите имя профиля notarytool (или нажмите Enter для пропуска): " NOTARYTOOL_PROFILE
        fi
        
        if [ -n "$NOTARYTOOL_PROFILE" ]; then
            echo ""
            echo "📤 Отправляю DMG на нотаризацию..."
            xcrun notarytool submit "$DMG_PATH" \
                --keychain-profile "$NOTARYTOOL_PROFILE" \
                --wait || {
                echo -e "${RED}❌ Ошибка нотаризации${NC}"
                exit 1
            }
            
            echo ""
            echo "📎 Скрепляю тикет..."
            xcrun stapler staple "$DMG_PATH" || {
                echo -e "${YELLOW}⚠️  Не удалось скрепить тикет${NC}"
            }
            
            echo -e "${GREEN}✅ DMG нотаризован${NC}"
        fi
    fi
fi

echo ""
echo -e "${GREEN}✅ Готово!${NC}"
echo ""
echo "Приложение: $APP_PATH"
if [ -f "$DMG_PATH" ]; then
    echo "DMG: $DMG_PATH"
fi
