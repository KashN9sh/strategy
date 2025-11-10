// Утилита для просмотра props.png и определения индексов спрайтов
// Запуск: cargo run --bin props_viewer

use anyhow::Result;
use image::GenericImageView;
use std::cell::Cell;
use std::sync::Arc;
use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ViewUniform {
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    padding: f32,
}

const SPRITE_SIZE: u32 = 16; // Размер одного спрайта в пикселях
const COLS: u32 = 5; // Количество колонок в атласе

fn main() -> Result<()> {
    env_logger::init();
    
    // Загружаем props.png
    let img = image::open("assets/props.png")?;
    let (img_width, img_height) = img.dimensions();
    let img_rgba = img.to_rgba8();
    
    println!("Загружен props.png: {}x{} пикселей", img_width, img_height);
    println!("Размер спрайта: {}x{}", SPRITE_SIZE, SPRITE_SIZE);
    println!("Колонок: {}, Строк: {}", COLS, img_height / SPRITE_SIZE);
    
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new()
        .with_title("Props Viewer - ЛКМ: индекс | ПКМ: перетаскивание | Колесо: масштаб | Стрелки: перемещение | Home: сброс")
        .with_inner_size(winit::dpi::PhysicalSize::new(img_width, img_height))
        .build(&event_loop)?);
    
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    let surface = instance.create_surface(window.clone())?;
    
    let adapter = pollster::block_on(
        instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
    ).ok_or_else(|| anyhow::anyhow!("Не удалось найти адаптер"))?;
    
    let (device, queue) = pollster::block_on(
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
    )?;
    
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    
    let size = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);
    
    // Создаем текстуру из изображения
    let texture_size = wgpu::Extent3d {
        width: img_width,
        height: img_height,
        depth_or_array_layers: 1,
    };
    
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Props Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img_rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * img_width),
            rows_per_image: Some(img_height),
        },
        texture_size,
    );
    
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Props Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    
    // Простой шейдер для отображения текстуры
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Props Viewer Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/props_viewer.wgsl").into()),
    });
    
    // Вершины для полноэкранного квада
    let vertices: &[f32] = &[
        // Позиция (x, y), UV (u, v)
        -1.0, -1.0,  0.0, 1.0,  // левый нижний
         1.0, -1.0,  1.0, 1.0,  // правый нижний
         1.0,  1.0,  1.0, 0.0,  // правый верхний
        -1.0,  1.0,  0.0, 0.0,  // левый верхний
    ];
    
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    // Создаем uniform buffer для масштаба и смещения
    let view_uniform = Cell::new(ViewUniform {
        zoom: 1.0,
        offset_x: 0.0,
        offset_y: 0.0,
        padding: 0.0,
    });
    
    let view_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("View Uniform Buffer"),
        contents: bytemuck::cast_slice(&[view_uniform.get()]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    
    let view_uniform_ref = &view_uniform;
    
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: view_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 4 * 4, // 4 float32
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2, // позиция
                    },
                    wgpu::VertexAttribute {
                        offset: 4 * 2,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2, // UV
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });
    
    let cursor_pos = Cell::new(winit::dpi::PhysicalPosition::new(0.0, 0.0));
    let cursor_pos_ref = &cursor_pos;
    let zoom = Cell::new(1.0f32);
    let zoom_ref = &zoom;
    let is_dragging = Cell::new(false);
    let is_dragging_ref = &is_dragging;
    let drag_start_pos = Cell::new(winit::dpi::PhysicalPosition::new(0.0, 0.0));
    let drag_start_pos_ref = &drag_start_pos;
    let drag_start_offset = Cell::new((0.0f32, 0.0f32));
    let drag_start_offset_ref = &drag_start_offset;
    
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    config.width = physical_size.width;
                    config.height = physical_size.height;
                    surface.configure(&device, &config);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                        let output = match surface.get_current_texture() {
                            Ok(texture) => texture,
                            Err(e) => {
                                eprintln!("Ошибка получения текстуры: {:?}", e);
                                return;
                            }
                        };
                        
                        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        
                        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });
                        
                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.1,
                                            g: 0.1,
                                            b: 0.1,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                            
                            render_pass.set_pipeline(&render_pipeline);
                            render_pass.set_bind_group(0, &bind_group, &[]);
                            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(0..6, 0, 0..1);
                        }
                        
                        queue.submit(std::iter::once(encoder.finish()));
                        output.present();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    cursor_pos_ref.set(position);
                    
                    // Если перетаскиваем, обновляем смещение
                    if is_dragging_ref.get() {
                        let start_pos = drag_start_pos_ref.get();
                        let start_offset = drag_start_offset_ref.get();
                        let current_zoom = zoom_ref.get();
                        
                        // Вычисляем смещение в пикселях экрана
                        let dx = (position.x - start_pos.x) as f32;
                        let dy = (position.y - start_pos.y) as f32;
                        
                        // Преобразуем в смещение в UV координатах (с учетом масштаба)
                        // Чем больше zoom, тем меньше должно быть смещение в UV
                        let uv_dx = dx / (config.width as f32 * current_zoom);
                        let uv_dy = dy / (config.height as f32 * current_zoom);
                        
                        // Обновляем uniform
                        // Инвертируем только вертикальное направление: тянем вниз -> сдвигаем вверх
                        let mut uniform = view_uniform_ref.get();
                        uniform.offset_x = start_offset.0 - uv_dx; // Обычное: тянем вправо -> сдвигаем вправо
                        uniform.offset_y = start_offset.1 - uv_dy; // Инвертировано: тянем вниз -> offset уменьшается (сдвиг вверх)
                        view_uniform_ref.set(uniform);
                        queue.write_buffer(&view_uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
                        window.request_redraw();
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => if y > 0.0 { 1.1 } else { 0.9 },
                        MouseScrollDelta::PixelDelta(p) => if p.y > 0.0 { 1.1 } else { 0.9 },
                    };
                    let new_zoom = (zoom_ref.get() * factor).clamp(0.1, 10.0);
                    zoom_ref.set(new_zoom);
                    
                    // Обновляем uniform
                    let mut uniform = view_uniform_ref.get();
                    uniform.zoom = new_zoom;
                    view_uniform_ref.set(uniform);
                    queue.write_buffer(&view_uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
                    window.request_redraw();
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Right,
                    state: ElementState::Pressed,
                    ..
                } => {
                    // Начинаем перетаскивание правой кнопкой мыши
                    let pos = cursor_pos_ref.get();
                    drag_start_pos_ref.set(pos);
                    let uniform = view_uniform_ref.get();
                    drag_start_offset_ref.set((uniform.offset_x, uniform.offset_y));
                    is_dragging_ref.set(true);
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Right,
                    state: ElementState::Released,
                    ..
                } => {
                    // Заканчиваем перетаскивание
                    is_dragging_ref.set(false);
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                    ..
                } => {
                    let pos = cursor_pos_ref.get();
                    let current_zoom = zoom_ref.get();
                    let uniform = view_uniform_ref.get();
                    
                    // Преобразуем экранные координаты в UV координаты [0, 1]
                    // Вершины квада имеют UV от (0,1) в левом нижнем до (1,0) в правом верхнем
                    // Экранные координаты: (0,0) в левом верхнем, (width, height) в правом нижнем
                    let screen_x = pos.x as f32;
                    let screen_y = pos.y as f32;
                    let screen_w = config.width as f32;
                    let screen_h = config.height as f32;
                    
                    // Преобразуем экранные координаты в UV координаты вершин квада
                    // Вершины квада определены как:
                    // (-1, -1) -> UV (0, 1) - левый нижний
                    // (1, -1) -> UV (1, 1) - правый нижний
                    // (1, 1) -> UV (1, 0) - правый верхний
                    // (-1, 1) -> UV (0, 0) - левый верхний
                    // Так что в UV квада: Y=0 вверху, Y=1 внизу
                    // Экран: (0,0) левый верхний, (width, height) правый нижний
                    // X: от 0 до width -> от 0 до 1 (прямое соответствие)
                    // Y: от 0 (верх экрана) до height (низ экрана) -> от 0 (верх квада) до 1 (низ квада)
                    // НЕ инвертируем, так как в UV квада Y=0 уже вверху!
                    let base_uv_x = screen_x / screen_w;
                    let base_uv_y = screen_y / screen_h;
                    
                    // Применяем ту же логику, что и в шейдере:
                    // В шейдере: centered_uv = in.uv - 0.5
                    //            scaled_uv = centered_uv / zoom + 0.5
                    //            final_uv = scaled_uv + offset
                    // Где in.uv = base_uv (UV координаты вершин квада)
                    let centered_uv_x = base_uv_x - 0.5;
                    let centered_uv_y = base_uv_y - 0.5;
                    let scaled_uv_x = centered_uv_x / current_zoom + 0.5;
                    let scaled_uv_y = centered_uv_y / current_zoom + 0.5;
                    let final_uv_x = scaled_uv_x + uniform.offset_x;
                    let final_uv_y = scaled_uv_y + uniform.offset_y;
                    
                    // Преобразуем UV в пиксели текстуры
                    // В текстурах: X идет от 0 (слева) до width (справа) - прямое соответствие
                    //              Y идет от 0 (вверху) до height (внизу)
                    // В final_uv: X идет от 0 до 1 (слева направо) - прямое соответствие
                    //            Y идет от 0 до 1, где 0 = верх квада = верх текстуры (в textureSample Y=0 вверху)
                    // Поэтому НЕ нужно инвертировать: tex_y = final_uv_y * height
                    let tex_x = (final_uv_x * img_width as f32) as u32;
                    let tex_y = (final_uv_y * img_height as f32) as u32;
                    
                    // Вычисляем колонку и строку спрайта
                    let col = tex_x / SPRITE_SIZE;
                    let row = tex_y / SPRITE_SIZE;
                    
                    // Вычисляем индекс (col + row * cols)
                    let index = col + row * COLS;
                    
                    println!("\n=== Клик на спрайт ===");
                    println!("Позиция мыши: ({}, {})", screen_x, screen_y);
                    println!("Base UV: ({:.3}, {:.3})", base_uv_x, base_uv_y);
                    println!("Final UV: ({:.3}, {:.3})", final_uv_x, final_uv_y);
                    println!("Координаты текстуры: ({}, {})", tex_x, tex_y);
                    println!("Колонка: {}, Строка: {}", col, row);
                    println!("Индекс спрайта: {}", index);
                    println!("Формула: col + row * cols = {} + {} * {} = {}", col, row, COLS, index);
                    println!("Масштаб: {:.2}x, Offset: ({:.3}, {:.3})", current_zoom, uniform.offset_x, uniform.offset_y);
                    println!("=====================\n");
                }
                WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state: ElementState::Pressed, .. }, .. } => {
                    let mut uniform = view_uniform_ref.get();
                    let current_zoom = zoom_ref.get();
                    let move_speed = 0.1 / current_zoom; // Скорость перемещения зависит от масштаба
                    let mut needs_update = false;
                    
                    if let PhysicalKey::Code(key_code) = physical_key {
                        match key_code {
                            KeyCode::ArrowLeft => {
                                uniform.offset_x -= move_speed;
                                needs_update = true;
                            }
                            KeyCode::ArrowRight => {
                                uniform.offset_x += move_speed;
                                needs_update = true;
                            }
                            KeyCode::ArrowUp => {
                                uniform.offset_y -= move_speed;
                                needs_update = true;
                            }
                            KeyCode::ArrowDown => {
                                uniform.offset_y += move_speed;
                                needs_update = true;
                            }
                            KeyCode::Home => {
                                // Сброс позиции и масштаба
                                uniform.offset_x = 0.0;
                                uniform.offset_y = 0.0;
                                uniform.zoom = 1.0;
                                zoom_ref.set(1.0);
                                needs_update = true;
                            }
                            _ => {}
                        }
                    }
                    
                    if needs_update {
                        view_uniform_ref.set(uniform);
                        queue.write_buffer(&view_uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
                        window.request_redraw();
                    }
                }
                _ => {
                    window.request_redraw();
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}

