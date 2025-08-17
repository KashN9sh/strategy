use strategy::render::wgpu_renderer::WgpuRenderer;
use strategy::render::ui_renderer::{UIRenderer, UIElement, UIElementType};
use strategy::types::{Building, BuildingKind};
use glam::IVec2;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use std::sync::Arc;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    
    let mut renderer = pollster::block_on(WgpuRenderer::new(window.clone())).unwrap();
    
    // Создаем тестовую placeholder текстуру
    println!("Создаем тестовую текстуру...");
    let result = renderer.texture_manager.create_placeholder(
        &renderer.device,
        &renderer.queue,
        "test_texture",
        64,
        64,
        [255, 0, 255, 255], // Розовый цвет
    );
    
    match result {
        Ok(_) => println!("✅ Текстура создана успешно"),
        Err(e) => println!("❌ Ошибка создания текстуры: {:?}", e),
    }
    
    println!("Доступные текстуры: {:?}", renderer.texture_manager.list_textures());
    
    // Проверяем, что текстура существует
    if let Some(texture) = renderer.texture_manager.get_texture("test_texture") {
        println!("✅ Текстура найдена, размер: {}x{}", texture.size.width, texture.size.height);
    } else {
        println!("❌ Текстура не найдена!");
    }
    
    // Создаем tile renderer для карты 10x10
    println!("Создаем tile renderer для карты 10x10...");
    renderer.create_tile_renderer(10, 10, 0.1); // 10x10 тайлов, размер тайла 0.1
    println!("✅ Tile renderer создан!");
    
    // Создаем building renderer
    println!("Создаем building renderer...");
    renderer.create_building_renderer(0.1); // Размер тайла 0.1
    println!("✅ Building renderer создан!");
    
    // Создаем тестовые здания
    let test_buildings = vec![
        Building {
            kind: BuildingKind::House,
            pos: IVec2::new(2, 2),
            timer_ms: 0,
            workers_target: 0,
            capacity: 5,
        },
        Building {
            kind: BuildingKind::Lumberjack,
            pos: IVec2::new(3, 3),
            timer_ms: 0,
            workers_target: 2,
            capacity: 0,
        },
        Building {
            kind: BuildingKind::Warehouse,
            pos: IVec2::new(4, 4),
            timer_ms: 0,
            workers_target: 0,
            capacity: 0,
        },
    ];
    
    // Обновляем здания в renderer
    if let Some(ref mut building_renderer) = renderer.building_renderer {
        building_renderer.update_buildings(&renderer.device, test_buildings);
        println!("✅ Тестовые здания добавлены!");
    }
    
    // Создаем UI renderer
    println!("Создаем UI renderer...");
    renderer.create_ui_renderer(800.0, 600.0); // Размеры окна
    println!("✅ UI renderer создан!");
    
    // Создаем тестовые UI элементы
    let test_ui_elements = vec![
        UIRenderer::create_panel(10.0, 10.0, 200.0, 100.0), // Панель в левом верхнем углу
        UIRenderer::create_button(20.0, 20.0, 80.0, 30.0, "Кнопка".to_string()), // Кнопка на панели
        UIRenderer::create_text_element(20.0, 60.0, 180.0, 20.0, "Тестовый текст".to_string()), // Текст
    ];
    
    // Обновляем UI элементы в renderer
    if let Some(ref mut ui_renderer) = renderer.ui_renderer {
        ui_renderer.update_elements(&renderer.device, test_ui_elements);
        println!("✅ Тестовые UI элементы добавлены!");
    }
    
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                renderer.resize(physical_size);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    }).unwrap();
}
