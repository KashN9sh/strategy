use std::path::PathBuf;

/// Определяет базовый путь к ресурсам приложения
/// 
/// Для macOS bundle: ищет ресурсы в Contents/Resources/
/// Для Windows: ищет ресурсы рядом с .exe файлом
/// Для обычного запуска: использует текущую директорию
pub fn resource_base_path() -> PathBuf {
    // Получаем путь к исполняемому файлу
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Проверяем, запущены ли мы из .app bundle (macOS)
            #[cfg(target_os = "macos")]
            {
                // Если мы в Contents/MacOS/, то ресурсы в Contents/Resources/
                if exe_dir.ends_with("Contents/MacOS") {
                    if let Some(contents) = exe_dir.parent() {
                        let resources = contents.join("Resources");
                        if resources.exists() {
                            log::info!("Найдены ресурсы в macOS bundle: {:?}", resources);
                            return resources;
                        }
                    }
                }
                // Если мы в bundle, но не в MacOS, попробуем найти Resources
                if let Some(contents) = exe_dir.parent() {
                    if contents.ends_with("Contents") {
                        let resources = contents.join("Resources");
                        if resources.exists() {
                            log::info!("Найдены ресурсы в macOS bundle: {:?}", resources);
                            return resources;
                        }
                    }
                }
            }
            
            // Для Windows и других систем: ресурсы рядом с .exe
            // Проверяем наличие папки assets рядом с exe
            let assets_dir = exe_dir.join("assets");
            if assets_dir.exists() {
                log::info!("Найдены ресурсы рядом с exe: {:?}", exe_dir);
                return exe_dir.to_path_buf();
            }
            
            // Также проверяем родительскую директорию (для случаев, когда exe в подпапке)
            if let Some(parent) = exe_dir.parent() {
                let assets_dir = parent.join("assets");
                if assets_dir.exists() {
                    log::info!("Найдены ресурсы в родительской директории: {:?}", parent);
                    return parent.to_path_buf();
                }
            }
        }
    }
    
    // По умолчанию используем текущую директорию
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    log::info!("Используем текущую директорию для ресурсов: {:?}", current_dir);
    current_dir
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
/// Для Windows: %APPDATA%/Cozy Kingdom/
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
    
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let app_support = PathBuf::from(appdata).join("Cozy Kingdom");
            // Создаем директорию если её нет
            if let Err(_) = std::fs::create_dir_all(&app_support) {
                // Если не удалось создать, используем текущую директорию
                return PathBuf::from(".");
            }
            return app_support;
        }
    }
    
    // Для других систем или если переменные окружения не найдены
    // Используем текущую директорию
    PathBuf::from(".")
}
