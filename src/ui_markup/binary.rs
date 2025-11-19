// Сериализатор/десериализатор для бинарного формата UI (.uib)

use crate::ui_markup::ast::UITree;
use std::fs::File;
use std::io::{Read, Write};

/// Сохранить UI дерево в бинарный файл (.uib)
pub fn save_binary(tree: &UITree, path: &str) -> Result<(), String> {
    // Сериализуем в JSON, затем сжимаем (можно использовать MessagePack или bincode для лучшей производительности)
    let json = serde_json::to_string(tree)
        .map_err(|e| format!("Failed to serialize UI tree: {}", e))?;
    
    let mut file = File::create(path)
        .map_err(|e| format!("Failed to create file {}: {}", path, e))?;
    
    // Записываем версию формата
    file.write_all(b"UIB1")
        .map_err(|e| format!("Failed to write header: {}", e))?;
    
    // Записываем размер данных
    let data_len = json.len() as u32;
    file.write_all(&data_len.to_le_bytes())
        .map_err(|e| format!("Failed to write data length: {}", e))?;
    
    // Записываем данные
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write data: {}", e))?;
    
    Ok(())
}

/// Загрузить UI дерево из бинарного файла (.uib)
pub fn load_binary(path: &str) -> Result<UITree, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file {}: {}", path, e))?;
    
    // Читаем заголовок
    let mut header = [0u8; 4];
    file.read_exact(&mut header)
        .map_err(|e| format!("Failed to read header: {}", e))?;
    
    if &header != b"UIB1" {
        return Err("Invalid file format".to_string());
    }
    
    // Читаем размер данных
    let mut len_bytes = [0u8; 4];
    file.read_exact(&mut len_bytes)
        .map_err(|e| format!("Failed to read data length: {}", e))?;
    let data_len = u32::from_le_bytes(len_bytes) as usize;
    
    // Читаем данные
    let mut data = vec![0u8; data_len];
    file.read_exact(&mut data)
        .map_err(|e| format!("Failed to read data: {}", e))?;
    
    // Десериализуем
    let json = String::from_utf8(data)
        .map_err(|e| format!("Failed to decode data: {}", e))?;
    
    let tree: UITree = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to deserialize UI tree: {}", e))?;
    
    Ok(tree)
}

/// Компилировать .ui файл в .uib
pub fn compile_ui_to_binary(ui_path: &str, uib_path: &str) -> Result<(), String> {
    let tree = crate::ui_markup::load_ui_from_file(ui_path)?;
    save_binary(&tree, uib_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_markup::ast::*;
    
    #[test]
    fn test_binary_format() {
        let tree = UITree::new();
        
        // Тест сериализации
        let result = save_binary(&tree, "/tmp/test.uib");
        assert!(result.is_ok());
        
        // Тест десериализации
        let loaded = load_binary("/tmp/test.uib");
        assert!(loaded.is_ok());
        
        // Очистка
        let _ = std::fs::remove_file("/tmp/test.uib");
    }
}
