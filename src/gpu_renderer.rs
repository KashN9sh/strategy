use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use glam::{Mat4, Vec2, Vec3};
use bytemuck::{Pod, Zeroable};
use anyhow::Result;

use crate::atlas::TileAtlas;
use crate::world::World;
use crate::types::TileKind;

// Структуры для передачи данных в GPU

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 2],
    zoom: f32,
    padding: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view_position: [0.0, 0.0],
            zoom: 1.0,
            padding: 0.0,
        }
    }

    pub fn update_view_proj(&mut self, view_proj: Mat4) {
        self.view_proj = view_proj.to_cols_array_2d();
    }

    pub fn update_view_position(&mut self, pos: Vec2) {
        self.view_position = pos.to_array();
    }

    pub fn update_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = [
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x2,
        },
    ];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TileInstance {
    model_matrix: [[f32; 4]; 4],
    tile_id: u32,
    tint_color: [f32; 4],
    padding: [u32; 3], // выравнивание до 16 байт
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BuildingInstance {
    model_matrix: [[f32; 4]; 4],
    building_id: u32,
    tint_color: [f32; 4],
    padding: [u32; 3], // выравнивание до 16 байт
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UIRect {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
}


impl TileInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = [
        // model_matrix
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
            shader_location: 4,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
            shader_location: 5,
            format: wgpu::VertexFormat::Float32x4,
        },
        // tile_id
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
            shader_location: 6,
            format: wgpu::VertexFormat::Uint32,
        },
        // tint_color
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 4 + std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            shader_location: 7,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TileInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl UIRect {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = [
        // model_matrix (4 vec4)
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
            shader_location: 4,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
            shader_location: 5,
            format: wgpu::VertexFormat::Float32x4,
        },
        // color
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
            shader_location: 6,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIRect>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl BuildingInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = [
        // model_matrix
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
            shader_location: 4,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
            shader_location: 5,
            format: wgpu::VertexFormat::Float32x4,
        },
        // building_id
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
            shader_location: 6,
            format: wgpu::VertexFormat::Uint32,
        },
        // tint_color
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 4]>() * 4 + std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            shader_location: 7,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BuildingInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UIVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

impl UIVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = [
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: (std::mem::size_of::<[f32; 2]>() * 2) as wgpu::BufferAddress,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct GpuRenderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    // Шейдеры и пайплайны
    tile_render_pipeline: wgpu::RenderPipeline,
    building_render_pipeline: wgpu::RenderPipeline,
    ui_render_pipeline: wgpu::RenderPipeline,
    ui_rect_render_pipeline: wgpu::RenderPipeline,
    
    // Буферы
    tile_vertex_buffer: wgpu::Buffer,
    tile_index_buffer: wgpu::Buffer,
    building_vertex_buffer: wgpu::Buffer,
    building_index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    
    // Униформы
    camera_uniform: CameraUniform,
    
    // Текстуры
    texture_bind_group: Option<wgpu::BindGroup>,
    
    // Временные буферы для инстансов
    tile_instances: Vec<TileInstance>,
    building_instances: Vec<BuildingInstance>,
    ui_rects: Vec<UIRect>,
    max_instances: usize,
    tile_instance_buffer: wgpu::Buffer,
    building_instance_buffer: wgpu::Buffer,
    ui_rect_buffer: wgpu::Buffer,
}

impl GpuRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        
        // Создаём экземпляр WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone())?;
        
        // Запрашиваем адаптер
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Не удалось найти подходящий адаптер"))?;
        
        // Получаем устройство и очередь команд
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;
        
        // Настройка поверхности
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
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
        
        // Загружаем шейдеры
        let tile_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tile Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/tile.wgsl").into()),
        });
        
        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ui.wgsl").into()),
        });
        
        // Создаём шейдер для UI прямоугольников
        let ui_rect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ui_rect.wgsl").into()),
        });
        
        // Создаём униформы камеры
        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Загружаем спрайтшит
        let spritesheet_bytes = include_bytes!("../assets/spritesheet.png");
        let spritesheet_image = image::load_from_memory(spritesheet_bytes)?;
        let spritesheet_rgba = spritesheet_image.to_rgba8();
        
        let (spritesheet_width, spritesheet_height) = spritesheet_rgba.dimensions();
        
        let spritesheet_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spritesheet Texture"),
            size: wgpu::Extent3d {
                width: spritesheet_width,
                height: spritesheet_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &spritesheet_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &spritesheet_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * spritesheet_width),
                rows_per_image: Some(spritesheet_height),
            },
            wgpu::Extent3d {
                width: spritesheet_width,
                height: spritesheet_height,
                depth_or_array_layers: 1,
            },
        );
        
        let spritesheet_view = spritesheet_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let spritesheet_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // для пиксельной графики
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });
        
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        
        // Создаём bind group layout для текстур
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&spritesheet_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&spritesheet_sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });
        
        // Квадратная геометрия с правильными UV (изометрия через позиционирование)
        let vertices = [
            Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] },  // левый верхний
            Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] },   // правый верхний
            Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] },    // правый нижний
            Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] },   // левый нижний
        ];
        
        let indices: &[u16] = &[
            0, 1, 2,  // верхний треугольник
            0, 2, 3,  // нижний треугольник
        ];
        
        let tile_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let tile_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Геометрия для зданий (более высокий quad)
        let building_vertices = [
            Vertex { position: [-0.5, -1.5, 0.0], tex_coords: [0.0, 1.0] },  // левый верхний
            Vertex { position: [0.5, -1.5, 0.0], tex_coords: [1.0, 1.0] },   // правый верхний
            Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] },    // правый нижний
            Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] },   // левый нижний
        ];
        
        let building_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Vertex Buffer"),
            contents: bytemuck::cast_slice(&building_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let building_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Index Buffer"),
            contents: bytemuck::cast_slice(indices), // используем те же индексы
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Буферы для инстансов
        let max_instances = 10000;
        let tile_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tile Instance Buffer"),
            size: (std::mem::size_of::<TileInstance>() * max_instances) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let building_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Building Instance Buffer"),
            size: (std::mem::size_of::<BuildingInstance>() * max_instances) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let ui_rect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Rect Buffer"),
            size: (std::mem::size_of::<UIRect>() * max_instances) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        

        // Создаём render pipeline для тайлов с текстурами
        let tile_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Tile Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let tile_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tile Render Pipeline"),
            layout: Some(&tile_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &tile_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), TileInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &tile_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
        
        // Создаём render pipeline для зданий
        let building_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Building Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/building.wgsl").into()),
        });
        
        let building_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Building Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let building_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Building Render Pipeline"),
            layout: Some(&building_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &building_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), BuildingInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &building_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
        
        // Создаём render pipeline для UI (пока заглушка)
        let ui_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // UI Rect пайплайн (только с камерой, без текстур)
        let ui_rect_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Rect Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let ui_rect_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Rect Render Pipeline"),
            layout: Some(&ui_rect_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_rect_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), UIRect::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_rect_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
        
        // ui_render_pipeline - заглушка, используем только ui_rect_render_pipeline  
        let ui_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Render Pipeline Stub"),
            layout: Some(&ui_rect_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_rect_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), UIRect::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_rect_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
        
        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            tile_render_pipeline,
            building_render_pipeline,
            ui_render_pipeline,
            ui_rect_render_pipeline,
            tile_vertex_buffer,
            tile_index_buffer,
            building_vertex_buffer,
            building_index_buffer,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            texture_bind_group: Some(texture_bind_group),
            tile_instances: Vec::new(),
            building_instances: Vec::new(),
            ui_rects: Vec::new(),
            max_instances,
            tile_instance_buffer,
            building_instance_buffer,
            ui_rect_buffer,
        })
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    // Обновление камеры с масштабированием как в CPU версии
    pub fn update_camera(&mut self, cam_x: f32, cam_y: f32, zoom: f32) {
        let aspect = self.size.width as f32 / self.size.height as f32;
        
        // Базируемся на размере экрана в пикселях, как в CPU версии
        let screen_width = self.size.width as f32;
        let screen_height = self.size.height as f32;
        
        // Ортогональная проекция в пикселях экрана (как в CPU)
        let ortho_size_x = screen_width / (2.0 * zoom);
        let ortho_size_y = screen_height / (2.0 * zoom);
        
        let projection = Mat4::orthographic_rh(
            -ortho_size_x, ortho_size_x,
            -ortho_size_y, ortho_size_y,
            -100.0, 100.0
        );
        
        // Камера сдвигается в пиксельных координатах
        let view = Mat4::from_translation(Vec3::new(-cam_x, cam_y, 0.0));
        let view_proj = projection * view;
        
        self.camera_uniform.update_view_proj(view_proj);
        self.camera_uniform.update_view_position(Vec2::new(cam_x, cam_y));
        self.camera_uniform.update_zoom(zoom);
        
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform])
        );
    }
    
    // Подготовка тайлов для рендеринга (пиксельные координаты как в CPU)
    pub fn prepare_tiles(&mut self, world: &mut World, atlas: &crate::atlas::TileAtlas, min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32) {
        self.tile_instances.clear();
        
        // Пиксельные размеры как в CPU версии
        let half_w = atlas.half_w as f32;
        let half_h = atlas.half_h as f32;
        let tile_w_px = atlas.half_w * 2 + 1; // точная ширина тайла в пикселях
        
        for my in min_ty..=max_ty {
            for mx in min_tx..=max_tx {
                let kind = world.get_tile(mx, my);
                
                // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ как в CPU версии:
                // world_to_screen: sx = (mx - my) * half_w, sy = (mx + my) * half_h  
                let iso_x = (mx - my) as f32 * half_w;
                let iso_y = (mx + my) as f32 * half_h;
                
                // Размер тайла в пикселях (не произвольный!)
                let tile_size = tile_w_px as f32;
                
                let model_matrix = Mat4::from_translation(Vec3::new(iso_x, -iso_y, 0.0)) * 
                                   Mat4::from_scale(Vec3::new(tile_size, tile_size, 1.0));
                
                let tile_id = match kind {
                    TileKind::Grass => 0,
                    TileKind::Forest => 1,
                    TileKind::Water => 2,
                };
                
                let tint = [1.0, 1.0, 1.0, 1.0]; // без тинта пока
                
                self.tile_instances.push(TileInstance {
                    model_matrix: model_matrix.to_cols_array_2d(),
                    tile_id,
                    tint_color: tint,
                    padding: [0; 3],
                });
                
                // Лимит инстансов для производительности
                if self.tile_instances.len() >= 10000 {
                    break;
                }
            }
            if self.tile_instances.len() >= 10000 {
                break;
            }
        }
        
        // Загружаем данные инстансов в буфер
        if !self.tile_instances.is_empty() {
            self.queue.write_buffer(
                &self.tile_instance_buffer,
                0,
                bytemuck::cast_slice(&self.tile_instances)
            );
        }
    }
    
    // Функции для UI рендеринга
    pub fn clear_ui(&mut self) {
        self.ui_rects.clear();
    }
    
    pub fn add_ui_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        // Создаем матрицу трансформации для прямоугольника в экранных координатах
        let model_matrix = Mat4::from_scale_rotation_translation(
            Vec3::new(width, height, 1.0),
            glam::Quat::IDENTITY,
            Vec3::new(x + width * 0.5, y + height * 0.5, 0.0), // центрируем
        );
        
        let ui_rect = UIRect {
            model_matrix: model_matrix.to_cols_array_2d(),
            color,
        };
        
        self.ui_rects.push(ui_rect);
    }
    
    pub fn draw_ui_panel(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // Полупрозрачная темная панель
        self.add_ui_rect(x, y, width, height, [0.0, 0.0, 0.0, 0.8]);
    }
    
    pub fn draw_ui_resource_icon(&mut self, x: f32, y: f32, size: f32, color: [f32; 4]) {
        // Цветная иконка ресурса
        self.add_ui_rect(x, y, size, size, color);
    }

    // Подготовка структур (здания и деревья) для рендеринга с правильной сортировкой
    pub fn prepare_structures(
        &mut self, 
        world: &mut crate::world::World, 
        buildings: &Vec<crate::types::Building>,
        _building_atlas: &Option<crate::atlas::BuildingAtlas>,
        _tree_atlas: &Option<crate::atlas::TreeAtlas>,
        tile_atlas: &crate::atlas::TileAtlas,
        min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
    ) {
        use crate::types::BuildingKind;
        use glam::{Mat4, IVec2};
        use std::collections::HashMap;
        
        self.building_instances.clear();
        
        // Создаем хеш-карту для быстрого поиска зданий по позиции
        let mut buildings_by_pos: HashMap<(i32, i32), usize> = HashMap::new();
        for (i, b) in buildings.iter().enumerate() {
            buildings_by_pos.insert((b.pos.x, b.pos.y), i);
        }
        
        // Диагональный проход (как в draw_structures_diagonal_scan)
        let min_s = min_tx + min_ty;
        let max_s = max_tx + max_ty;
        
        // Пиксельные размеры как у тайлов (синхронно с CPU версией)
        let half_w = tile_atlas.half_w as f32;  
        let half_h = tile_atlas.half_h as f32;
        let tile_w_px = tile_atlas.half_w * 2 + 1;
        let building_size = tile_w_px as f32; // размер здания = размеру тайла
        
        for s in min_s..=max_s {
            for mx in min_tx..=max_tx {
                let my = s - mx;
                if my < min_ty || my > max_ty { continue; }
                
                // Сначала деревья (рисуем раньше для правильного порядка)
                if world.has_tree(IVec2::new(mx, my)) {
                    // TODO: Добавить рендеринг деревьев
                    // let stage = world.tree_stage(IVec2::new(mx, my)).unwrap_or(2) as usize;
                }
                
                // Затем здания
                if let Some(&building_idx) = buildings_by_pos.get(&(mx, my)) {
                    let building = &buildings[building_idx];
                    
                    // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ (как в CPU версии)
                    let iso_x = (mx - my) as f32 * half_w;
                    let iso_y = (mx + my) as f32 * half_h;
                    
                    // Смещение здания вниз как в CPU версии
                    let building_off = (half_h * 0.7); 
                    let final_y = iso_y + building_off;
                    
                    // Матрица трансформации здания (пиксельные координаты)
                    let transform = Mat4::from_scale_rotation_translation(
                        glam::Vec3::new(building_size, building_size, 1.0), // квадратные здания как тайлы
                        glam::Quat::IDENTITY,
                        glam::Vec3::new(iso_x, -final_y, 0.0) // используем final_y с building_off
                    );
                    
                    // Конвертируем BuildingKind в u32 ID
                    let building_id = match building.kind {
                        BuildingKind::House => 0,
                        BuildingKind::Lumberjack => 1,
                        BuildingKind::Warehouse => 2,
                        BuildingKind::Forester => 3,
                        BuildingKind::StoneQuarry => 4,
                        BuildingKind::ClayPit => 5,
                        BuildingKind::Kiln => 6,
                        BuildingKind::WheatField => 7,
                        BuildingKind::Mill => 8,
                        BuildingKind::Bakery => 9,
                        BuildingKind::Fishery => 10,
                        BuildingKind::IronMine => 11,
                        BuildingKind::Smelter => 12,
                    };
                    
                    let instance = BuildingInstance {
                        model_matrix: transform.to_cols_array_2d(),
                        building_id,
                        tint_color: [1.0, 1.0, 1.0, 1.0], // белый цвет по умолчанию
                        padding: [0; 3],
                    };
                    
                    self.building_instances.push(instance);
                }
            }
        }
        
        // Обновляем буфер инстансов зданий
        if !self.building_instances.is_empty() {
            self.queue.write_buffer(
                &self.building_instance_buffer,
                0,
                bytemuck::cast_slice(&self.building_instances)
            );
        }
    }
    
    // Основная функция рендеринга
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            // Рендерим тайлы с текстурами
            if let Some(ref texture_bind_group) = self.texture_bind_group {
                if !self.tile_instances.is_empty() {
                    render_pass.set_pipeline(&self.tile_render_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.tile_vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, self.tile_instance_buffer.slice(..));
                    render_pass.set_index_buffer(self.tile_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..self.tile_instances.len() as u32);
                }
                
                // Рендерим здания
                if !self.building_instances.is_empty() {
                    render_pass.set_pipeline(&self.building_render_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.building_vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, self.building_instance_buffer.slice(..));
                    render_pass.set_index_buffer(self.building_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..self.building_instances.len() as u32);
                }
                
                // Рендерим UI прямоугольники
                if !self.ui_rects.is_empty() {
                    // Обновляем буфер UI инстансов
                    self.queue.write_buffer(
                        &self.ui_rect_buffer,
                        0,
                        bytemuck::cast_slice(&self.ui_rects)
                    );
                    
                    render_pass.set_pipeline(&self.ui_rect_render_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.tile_vertex_buffer.slice(..)); // используем тот же quad
                    render_pass.set_vertex_buffer(1, self.ui_rect_buffer.slice(..));
                    render_pass.set_index_buffer(self.tile_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..self.ui_rects.len() as u32);
                }
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
    // Загрузка текстуры (пока заглушка)
    pub fn load_texture_atlas(&mut self, atlas: &TileAtlas) -> Result<()> {
        // TODO: реализовать загрузку текстурного атласа
        // Пока создадим простую заглушку
        Ok(())
    }
}
