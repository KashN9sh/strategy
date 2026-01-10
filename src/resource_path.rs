use std::path::{Path, PathBuf};

/// Определяет базовый путь к ресурсам приложения
/// 
/// Для macOS bundle: ищет ресурсы в Contents/Resources/
/// Для обычного запуска: использует текущую директорию
pub fn resource_base_path() -> PathBuf {
    // Получаем путь к исполняемому файлу
    if let Ok(exe_path) = std::env::current_exe() {
        // Проверяем, запущены ли мы из .app bundle (macOS)
        if let Some(parent) = exe_path.parent() {
            // Если мы в Contents/MacOS/, то ресурсы в Contents/Resources/
            if parent.ends_with("Contents/MacOS") {
                if let Some(contents) = parent.parent() {
                    let resources = contents.join("Resources");
                    if resources.exists() {
                        return resources;
                    }
                }
            }
            // Если мы в bundle, но не в MacOS, попробуем найти Resources
            if let Some(contents) = parent.parent() {
                if contents.ends_with("Contents") {
                    let resources = contents.join("Resources");
                    if resources.exists() {
                        return resources;
                    }
                }
            }
        }
    }
    
    // По умолчанию используем текущую директорию
    PathBuf::from(".")
}

/// Получить путь к файлу ресурса
pub fn resource_path(relative_path: &str) -> PathBuf {
    resource_base_path().join(relative_path)
}

/// Получить путь к assets
pub fn assets_path() -> PathBuf {
    resource_base_path().join("assets")
}

/// Получить путь к shaders (для совместимости, хотя они встроены)
pub fn shaders_path() -> PathBuf {
    resource_base_path().join("shaders")
}

/// Получить директорию для сохранения данных пользователя
/// 
/// Для macOS: ~/Library/Application Support/Cozy Kingdom/
/// Для других систем: ~/.cozy-kingdom/ или текущая директория
pub fn user_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let app_support = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Cozy Kingdom");
            // Создаем директорию если её нет
            if let Err(_) = std::fs::create_dir_all(&app_support) {
                // Если не удалось создать, используем текущую директорию
                return PathBuf::from(".");
            }
            return app_support;
        }
    }
    
    // Для других систем или если HOME не найден
    // Используем текущую директорию
    PathBuf::from(".")
}
