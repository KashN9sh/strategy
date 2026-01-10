# Инструкция по сборке установщиков для Cozy Kingdom

## Предварительные требования

### Для macOS:
- macOS с установленным Xcode Command Line Tools
- Rust (установлен через rustup)

### Для Windows:
- Windows 10/11
- Rust (установлен через rustup)
- WiX Toolset (для создания .msi установщика) - опционально

## Установка инструментов

### 1. cargo-bundle (для macOS .app и Windows .msi)

```bash
cargo install cargo-bundle
```

### 2. WiX Toolset (только для Windows, опционально)

Скачайте и установите с https://wixtoolset.org/releases/

## Конфигурация проекта

Создайте файл `bundle.toml` в корне проекта (опционально, для кастомизации):

```toml
[macos]
minimum_system_version = "10.13"

[windows]
console = false
```

## Сборка для macOS

### 1. Сборка release версии

```bash
cargo build --release
```

### 2. Создание .app bundle

```bash
cargo bundle --release --target x86_64-apple-darwin  # Intel Mac
# или
cargo bundle --release --target aarch64-apple-darwin  # Apple Silicon
```

Результат будет в `target/release/bundle/osx/Cozy Kingdom.app`

### 3. Создание .dmg установщика (опционально)

Установите `create-dmg`:
```bash
brew install create-dmg
```

Создайте DMG:
```bash
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
```

## Сборка для Windows

### 1. Сборка release версии

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

### 2. Создание .msi установщика

```bash
cargo bundle --release --target x86_64-pc-windows-msvc
```

Результат будет в `target/release/bundle/msi/Cozy Kingdom_0.1.0_x64_en-US.msi`

### 3. Альтернатива: создание портативной версии

Просто скопируйте `target/release/strategy.exe` вместе с папками:
- `assets/`
- `shaders/`

И упакуйте в ZIP архив.

## Структура bundle

После выполнения `cargo bundle`, структура будет следующей:

```
target/release/bundle/
├── osx/
│   └── Cozy Kingdom.app/
│       ├── Contents/
│       │   ├── MacOS/
│       │   │   └── strategy
│       │   ├── Resources/
│       │   │   └── assets/  (автоматически копируется)
│       │   └── Info.plist
│
└── msi/
    └── Cozy Kingdom_0.1.0_x64_en-US.msi
```

## Важные замечания

1. **Ресурсы**: `cargo-bundle` автоматически копирует папку `assets/` если она находится в корне проекта
2. **Шейдеры**: Убедитесь, что `shaders/` также копируется (может потребоваться ручная настройка)
3. **Подписка кода (macOS)**: Для распространения через App Store или Gatekeeper нужна подпись:
   ```bash
   codesign --deep --force --verify --verbose --sign "Developer ID Application: Your Name" "Cozy Kingdom.app"
   ```
4. **Notarization (macOS)**: Для распространения вне App Store нужна нотаризация через Apple

## Автоматизация через скрипты

В проекте уже есть готовые скрипты для автоматизации:

### macOS
```bash
./build-macos.sh
```

Скрипт автоматически:
- Определяет архитектуру (Intel/Apple Silicon)
- Собирает release версию
- Создает .app bundle
- Опционально создает DMG (если установлен create-dmg)

### Windows
```bash
./build-windows.sh
```

Скрипт автоматически:
- Устанавливает Windows target если нужно
- Собирает release версию
- Создает .msi установщик
- Опционально создает портативный ZIP пакет

## Быстрый старт

### macOS:
```bash
# Установить cargo-bundle
cargo install cargo-bundle

# Собрать
./build-macos.sh
```

### Windows (через WSL или Git Bash):
```bash
# Установить cargo-bundle
cargo install cargo-bundle

# Добавить Windows target
rustup target add x86_64-pc-windows-msvc

# Собрать
./build-windows.sh
```
