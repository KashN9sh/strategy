# UI Markup System

Декларативная система разметки UI для игры.

## Структура модуля

```
ui_markup/
├── mod.rs           - Публичный API
├── ast.rs           - AST для представления UI дерева
├── lexer.rs         - Токенизатор
├── parser.rs        - Парсер (indent-based)
├── components.rs    - Базовые UI компоненты
├── layout.rs        - Система лейаута (flexbox-подобная)
├── event.rs         - Система событий
├── context.rs       - Контекст рендеринга с доступом к игровому состоянию
├── renderer.rs      - Рендерер (преобразует UI дерево в команды GPU)
├── bindings.rs      - Система реактивных биндингов
├── theme.rs         - Система тематизации
├── binary.rs        - Сериализатор/десериализатор (.uib формат)
└── integration.rs   - Интеграция с игрой
```

## Быстрый старт

### 1. Создание UI файла (assets/ui/example.ui)

```ui
ui {
  panel #main background=#2c3e50 padding=16 {
    vbox gap=12 {
      text text="Hello World" color=#ffffff scale=1.5
      button text="Click Me" onclick="my_command"
    }
  }
}
```

### 2. Использование в коде

```rust
use strategy::ui_markup::UIMarkupManager;

// Инициализация
let mut ui_manager = UIMarkupManager::new(800.0, 600.0);
ui_manager.initialize()?;

// Рендеринг
ui_manager.render(&mut gpu_renderer, &game_state);

// Обработка событий
if let Some(command) = ui_manager.handle_click(x, y) {
    execute_ui_command(&command, &mut game_state);
}
```

## Примеры

### Базовая панель с ресурсами

```ui
ui {
  panel #top_bar position=top height=auto background=#1a1a1acc padding=8 {
    hbox gap=8 {
      icon sprite="props:1" size=12
      number bind="resources.gold" color=#ffcc00
      
      icon sprite="props:9" size=12
      number bind="resources.wood" color=#ffffff
    }
  }
}
```

### Условный рендеринг

```ui
ui {
  conditional if="paused" {
    text text="PAUSED" color=#ff0000 position=fixed x=400 y=50
  }
  
  conditional if="population > 0" {
    panel #population_info {
      number bind="population"
    }
  }
}
```

### Вкладки с кнопками

```ui
ui {
  vbox gap=8 {
    hbox #tabs gap=6 {
      button text="Build" onclick="switch_tab:build" active="ui_tab==Build"
      button text="Economy" onclick="switch_tab:economy" active="ui_tab==Economy"
    }
    
    conditional if="ui_tab==Build" {
      hbox #buildings gap=6 {
        button text="House" onclick="select_building:house"
        button text="Warehouse" onclick="select_building:warehouse"
      }
    }
  }
}
```

## API Reference

### Основные типы

- `UITree` - дерево UI элементов
- `UINode` - узел дерева (компонент)
- `ComponentType` - тип компонента (Panel, Button, Text и т.д.)
- `AttributeValue` - значение атрибута (String, Number, Bool, Color, Binding)
- `LayoutEngine` - вычисление позиций и размеров
- `RenderContext` - доступ к игровым данным
- `EventSystem` - обработка событий

### Функции

- `load_ui_from_file(path)` - загрузить UI из .ui файла
- `load_ui_from_binary(path)` - загрузить UI из .uib файла
- `save_ui_to_binary(tree, path)` - сохранить UI в .uib файл
- `parse_ui(markup)` - парсить UI из строки

## Тестирование

```bash
# Запустить все тесты модуля
cargo test ui_markup

# Запустить редактор UI
cargo run --bin ui_editor
```

## Performance

- Layout пересчитывается только при изменении размера окна или структуры UI
- Биндинги обновляются только при изменении данных (dirty flag pattern)
- Binary формат (.uib) загружается в ~10x быстрее текстового (.ui)
- GPU batching минимизирует draw calls

## Roadmap

- [ ] Анимация переходов
- [ ] Responsive breakpoints
- [ ] Drag & drop в runtime
- [ ] Scroll containers
- [ ] Grid layout (Row/Col)
- [ ] Кастомные шрифты
- [ ] Image компонент
- [ ] Video/Animation компонент

## Contributing

При добавлении новых компонентов:

1. Добавьте тип в `ComponentType` enum (ast.rs)
2. Реализуйте структуру компонента (components.rs)
3. Добавьте рендеринг в `UIMarkupRenderer` (renderer.rs)
4. Добавьте layout логику если нужно (layout.rs)
5. Обновите документацию

## License

Часть проекта strategy game.
