# Система разметки UI - Руководство

## Обзор

Система разметки UI предоставляет декларативный способ создания пользовательских интерфейсов для игры. Вместо императивного кода с явным указанием координат и размеров, вы описываете структуру UI в текстовых файлах с использованием простого indent-based синтаксиса.

## Преимущества

- ✅ **Декларативный подход** - описываете что нужно, а не как это сделать
- ✅ **Быстрая итерация** - изменения в UI файлах не требуют перекомпиляции
- ✅ **Hot-reload** - изменения применяются мгновенно в dev режиме
- ✅ **Визуальный редактор** - редактируйте UI визуально без написания кода
- ✅ **Реактивные биндинги** - UI автоматически обновляется при изменении данных
- ✅ **Компактный бинарный формат** - быстрая загрузка в production
- ✅ **Bootstrap-подобные компоненты** - готовая библиотека UI элементов

## Синтаксис разметки

### Базовая структура

```ui
ui scale=auto {
  panel #main_panel background=#2c3e50 padding=16 {
    button text="Click Me" onclick="do_something"
  }
}
```

### Элементы синтаксиса

- **Отступы** - определяют вложенность (как в Python/YAML)
- **#id** - указание идентификатора элемента
- **key=value** - атрибуты элемента
- **{}** - явная группировка (опционально)
- **//** - комментарии

### Типы значений

```ui
// Строки (с кавычками или без)
text="Hello World"
name=MyElement

// Числа
width=100
height=50.5
opacity=0.8

// Булевы значения
visible=true
active=false

// Цвета (hex формат)
color=#FF0000
background=#3498dbcc  // с альфа-каналом

// Биндинги к данным игры
text bind="resources.gold"
progress bind="day_progress"
```

## Компоненты

### Layout контейнеры

#### Panel - базовый контейнер

```ui
panel #my_panel background=#1a1a1a padding=8 border-color=#ffffff border-width=2 {
  // содержимое
}
```

**Атрибуты:**
- `background` - цвет фона
- `border-color` - цвет рамки
- `border-width` - толщина рамки
- `padding` - внутренние отступы

#### HBox - горизонтальный layout

```ui
hbox gap=8 align=center justify=space-between {
  button text="Button 1"
  button text="Button 2"
  button text="Button 3"
}
```

**Атрибуты:**
- `gap` - расстояние между элементами
- `align` - выравнивание по вертикали (start/center/end/stretch)
- `justify` - распределение по горизонтали (start/center/end/space-between/space-around)

#### VBox - вертикальный layout

```ui
vbox gap=12 {
  text text="Title"
  text text="Subtitle"
  button text="Action"
}
```

**Атрибуты:** аналогичны HBox

### UI элементы

#### Button - кнопка

```ui
button text="Click Me" onclick="my_command" active=false
```

**Атрибуты:**
- `text` - текст на кнопке
- `onclick` - команда при клике
- `active` - активна ли кнопка (подсвечена)

#### Text - текстовый элемент

```ui
// Статический текст
text text="Hello World" color=#ffffff scale=1.2

// Биндинг к данным
text bind="population" color=#ffffff
```

**Атрибуты:**
- `text` - статический текст
- `bind` - биндинг к данным игры
- `color` - цвет текста
- `scale` - масштаб текста

#### Number - отображение чисел

```ui
number bind="resources.gold" color=#ffcc00 scale=1.0
```

**Атрибуты:**
- `bind` - биндинг к числовым данным
- `color` - цвет
- `scale` - масштаб

#### Icon - иконка из спрайт-атласа

```ui
icon sprite="props:0" size=12
```

**Атрибуты:**
- `sprite` - идентификатор спрайта (формат "atlas:index")
- `size` - размер иконки

#### ProgressBar - прогресс бар

```ui
progressbar bind="day_progress" color=#4caf50 background=#1a1a1a height=8
```

**Атрибуты:**
- `progress` - значение прогресса (0.0 - 1.0) или биндинг
- `color` - цвет заполнения
- `background` - цвет фона
- `height` - высота бара

### Условный рендеринг

```ui
conditional if="population > 0" {
  text text="Population is not zero"
}

conditional if="has_research_lab" {
  button text="Research" onclick="open_research"
}
```

## Система биндингов

### Доступ к игровым данным

```ui
// Ресурсы
number bind="resources.gold"
number bind="resources.wood"

// Население
number bind="population"
number bind="happiness"

// Состояние игры
text bind="paused"
progressbar bind="day_progress"
```

### Условные выражения

```ui
conditional if="population > 10" {
  // показывается только если население больше 10
}

conditional if="paused == true" {
  text text="PAUSED" color=#ff0000
}

conditional if="resources.gold >= 100" {
  button text="Expensive Action"
}
```

## Система событий

### Обработчики событий

```ui
// Клик
button text="Build" onclick="switch_tab:build"

// Hover (для тултипов)
panel onhover="show_tooltip:info" {
  text text="Hover me"
}
```

### Команды

Команды имеют формат: `command_name` или `command_name:arg1,arg2`

**Встроенные команды:**

- `switch_tab:build` - переключить вкладку на Build
- `switch_tab:economy` - переключить вкладку на Economy
- `select_category:housing` - выбрать категорию Housing
- `toggle_deposits` - показать/скрыть депозиты ресурсов
- `toggle_research` - открыть/закрыть окно исследований
- `decrease_tax` - уменьшить налоги
- `increase_tax` - увеличить налоги
- `set_food_policy:balanced` - установить политику еды

## Тематизация

### Файл темы (theme.ui)

```ui
theme {
  colors {
    primary=#3498db
    success=#2ecc71
    danger=#e74c3c
    warning=#f39c12
    info=#1abc9c
    dark=#2c3e50
    light=#ecf0f1
  }
  
  spacing {
    base=8
    scale=[0, 8, 16, 24, 32, 40]
  }
  
  typography {
    base_scale=1.0
  }
  
  breakpoints {
    sm=576
    md=768
    lg=1024
    xl=1440
  }
}
```

### Использование цветов темы

```ui
panel background=primary {
  button text="Success" color=success
  button text="Danger" color=danger
}
```

## Интеграция с кодом

### Инициализация

```rust
use strategy::ui_markup::UIMarkupManager;

let mut ui_manager = UIMarkupManager::new(800.0, 600.0);
ui_manager.initialize()?;
```

### Рендеринг

```rust
// В игровом цикле
ui_manager.render(&mut gpu_renderer, &game_state);
```

### Обработка событий

```rust
// Клик мыши
if let Some(command) = ui_manager.handle_click(x, y) {
    execute_ui_command(&command, &mut game_state);
}

// Движение мыши (для hover)
ui_manager.handle_mouse_move(x, y);
```

### Hot-reload (dev режим)

```rust
// Перезагрузить UI из файлов
ui_manager.reload()?;
```

## Визуальный редактор

### Запуск редактора

```bash
cargo run --bin ui_editor
```

### Возможности редактора

- **Дерево компонентов** - визуальная иерархия UI элементов
- **Live preview** - предпросмотр с реальным рендером
- **Панель свойств** - редактирование атрибутов компонентов
- **Drag & Drop** - перетаскивание компонентов
- **Auto-complete** - подсказки для биндингов и команд
- **Валидация** - проверка ошибок в реальном времени
- **Экспорт в .uib** - компиляция в бинарный формат

## Примеры

### Простой UI

```ui
ui {
  panel #top_bar position=top background=#1a1a1a padding=8 {
    hbox gap=8 {
      icon sprite="props:1" size=12
      number bind="resources.gold" color=#ffcc00
    }
  }
}
```

### Кнопки с категориями

```ui
ui {
  panel #bottom_bar position=bottom {
    vbox gap=8 {
      hbox #tabs gap=6 {
        button text="Build" onclick="switch_tab:build" active="ui_tab==Build"
        button text="Economy" onclick="switch_tab:economy" active="ui_tab==Economy"
      }
      
      conditional if="ui_tab==Build" {
        hbox #categories gap=6 {
          button text="Housing" onclick="select_category:housing"
          button text="Food" onclick="select_category:food"
        }
      }
    }
  }
}
```

### Модальное окно

```ui
ui {
  conditional if="show_modal" {
    panel #overlay position=fixed x=0 y=0 width=100% height=100% background=#00000080 {
      panel #modal position=center width=400 height=300 background=#2c3e50 padding=16 {
        vbox gap=12 {
          text text="Modal Title" scale=1.5
          text text="Some content here"
          button text="Close" onclick="close_modal"
        }
      }
    }
  }
}
```

## Производительность

### Оптимизации

- **Layout caching** - layout пересчитывается только при изменениях
- **Dirty flag pattern** - обновляются только измененные элементы
- **Бинарный формат** - .uib файлы загружаются в 10x быстрее .ui
- **Batching** - UI команды группируются для GPU

### Рекомендации

1. Используйте `.uib` файлы в production сборке
2. Минимизируйте количество биндингов в сложных UI
3. Используйте `conditional` для скрытия неиспользуемых частей UI
4. Группируйте статические элементы в отдельные панели

## Миграция с императивного UI

### До (ui_gpu.rs)

```rust
gpu.draw_ui_panel(0.0, 0.0, width, height);
gpu.draw_text(x, y, b"Gold:", color, scale);
gpu.draw_number(x + 50, y, gold, color, scale);
```

### После (main.ui)

```ui
panel {
  hbox gap=8 {
    text text="Gold:"
    number bind="resources.gold"
  }
}
```

## Расширение системы

### Добавление кастомных компонентов

```rust
// В components.rs
pub struct CustomWidget {
    pub style: ComponentStyle,
    pub custom_data: String,
}

impl CustomWidget {
    pub fn from_node(node: &UINode) -> Self {
        CustomWidget {
            style: ComponentStyle::from_node(node),
            custom_data: node.get_string_attr("custom").unwrap_or("").to_string(),
        }
    }
}
```

### Добавление новых команд

```rust
// В integration.rs
match cmd {
    "my_custom_command" => {
        // Ваша логика здесь
        return true;
    }
    // ...
}
```

## Troubleshooting

### UI не отображается

- Проверьте, что `ui_manager.initialize()` вызван
- Убедитесь, что путь к .ui файлам корректен
- Проверьте синтаксис .ui файла

### Биндинги не обновляются

- Убедитесь, что `update_context()` вызывается каждый кадр
- Проверьте правильность пути биндинга (например, "resources.gold")
- Проверьте, что данные действительно изменяются

### События не обрабатываются

- Проверьте регистрацию обработчиков в `register_event_handlers()`
- Убедитесь, что layout вычислен корректно
- Проверьте, что элемент не перекрыт другими элементами (z-order)

## Дополнительные ресурсы

- [Примеры UI файлов](assets/ui/)
- [Исходный код ui_editor](src/bin/ui_editor.rs)
- [Документация API](src/ui_markup/)

## FAQ

**Q: Можно ли использовать систему разметки с другими играми?**
A: Да, система достаточно универсальна и может быть адаптирована.

**Q: Поддерживается ли анимация?**
A: Пока нет, но запланировано в будущих версиях.

**Q: Как создать сложные grid layouts?**
A: Используйте комбинацию HBox и VBox или добавьте компонент Row/Col.

**Q: Можно ли динамически создавать UI из кода?**
A: Да, UITree можно создавать и модифицировать программно.

**Q: Как добавить кастомные шрифты?**
A: Пока система использует встроенный bitmap шрифт, поддержка кастомных шрифтов планируется.
