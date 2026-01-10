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
