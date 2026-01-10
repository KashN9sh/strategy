# Подпись и нотаризация приложения Cozy Kingdom для macOS

## Предварительные требования

1. **Apple Developer аккаунт** ($99/год)
   - Зарегистрируйтесь на https://developer.apple.com
   - Оплатите годовую подписку

2. **Установите Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```

3. **Создайте сертификат разработчика**:
   - Откройте Xcode → Preferences → Accounts
   - Добавьте ваш Apple ID
   - Нажмите "Manage Certificates"
   - Создайте "Developer ID Application" сертификат (для распространения вне App Store)
   - Или "Apple Development" сертификат (для тестирования)

## Процесс подписи

### 1. Проверка сертификатов

```bash
security find-identity -v -p codesigning
```

Вы должны увидеть что-то вроде:
```
Developer ID Application: Your Name (TEAM_ID)
```

### 2. Подпись приложения

```bash
# Замените "Developer ID Application: Your Name" на ваш сертификат
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name" \
  "target/release/bundle/osx/Cozy Kingdom.app"
```

### 3. Проверка подписи

```bash
codesign --verify --verbose "target/release/bundle/osx/Cozy Kingdom.app"
spctl --assess --verbose "target/release/bundle/osx/Cozy Kingdom.app"
```

### 4. Нотаризация (для распространения вне App Store)

Нотаризация требуется для macOS 10.15+ при распространении вне App Store.

#### Создайте App-Specific Password:
1. Перейдите на https://appleid.apple.com
2. Войдите в аккаунт
3. В разделе "Security" → "App-Specific Passwords"
4. Создайте новый пароль для "notarytool"

#### Настройте notarytool:

```bash
xcrun notarytool store-credentials \
  --apple-id "your.email@example.com" \
  --team-id "YOUR_TEAM_ID" \
  --password "app-specific-password" \
  "notarytool-profile"
```

#### Нотаризуйте DMG:

```bash
xcrun notarytool submit "Cozy Kingdom.dmg" \
  --keychain-profile "notarytool-profile" \
  --wait
```

#### Скрепите тикет:

```bash
xcrun stapler staple "Cozy Kingdom.dmg"
```

## Автоматизация через скрипт

Используйте `sign-macos.sh` для автоматической подписи и нотаризации.

## Важные замечания

1. **Team ID**: Найдите его в Apple Developer Portal или через:
   ```bash
   security find-identity -v -p codesigning | grep "Developer ID"
   ```

2. **Entitlements**: Для некоторых функций может потребоваться файл entitlements:
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>com.apple.security.cs.allow-jit</key>
       <true/>
   </dict>
   </plist>
   ```

3. **Hardened Runtime**: Для macOS 10.14+ может потребоваться:
   ```bash
   codesign --options runtime --sign "..." "Cozy Kingdom.app"
   ```

4. **Проверка Gatekeeper**:
   ```bash
   spctl -a -vv "Cozy Kingdom.app"
   ```

## Альтернатива: Ad-hoc подпись (для тестирования)

Если у вас нет Apple Developer аккаунта, можно использовать ad-hoc подпись:

```bash
codesign --deep --force --sign "-" "target/release/bundle/osx/Cozy Kingdom.app"
```

Это позволит запускать приложение локально, но не пройдет Gatekeeper для других пользователей.
