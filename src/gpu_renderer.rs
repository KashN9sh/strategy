use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use glam::{Mat4, Vec2, Vec3};
use bytemuck::{Pod, Zeroable};
use anyhow::Result;

use crate::atlas::TileAtlas;
use crate::world::World;
use crate::types::{TileKind, WeatherKind, BuildingKind, BiomeKind};

// Структуры для передачи данных в GPU

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct WeatherUniform {
    pub weather_type: u32, // 0=Clear, 1=Rain, 2=Fog, 3=Snow
    pub time: f32,
    pub intensity: f32,
    pub padding: f32,
}

impl WeatherUniform {
    pub fn new() -> Self {
        Self {
            weather_type: 0,
            time: 0.0,
            intensity: 0.0,
            padding: 0.0,
        }
    }
}

// Структуры для частиц зданий
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct BuildingParticle {
    pub position: [f32; 2],    // Позиция частицы
    pub velocity: [f32; 2],    // Скорость частицы
    pub life: f32,             // Время жизни (0.0-1.0)
    pub size: f32,             // Размер частицы
    pub color: [f32; 4],       // Цвет частицы
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct BuildingParticleUniform {
    pub time: f32,
    pub particle_count: u32,
    pub padding: [f32; 2],
}

impl BuildingParticleUniform {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            particle_count: 0,
            padding: [0.0; 2],
        }
    }
}

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
pub struct RoadInstance {
    model_matrix: [[f32; 4]; 4],
    road_mask: u32, // битовая маска соединений (0-15)
    tint_color: [f32; 4],
    padding: [u32; 3], // выравнивание до 16 байт
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UIRect {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ScreenUniform {
    screen_size: [f32; 2],
    padding: [f32; 2],
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

impl RoadInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = [
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
        // road_mask
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
            shader_location: 6,
            format: wgpu::VertexFormat::Uint32,
        },
        // tint_color
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress + 4,
            shader_location: 7,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RoadInstance>() as wgpu::BufferAddress,
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

// Структура для рендеринга граждан (используем building shader)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CitizenInstance {
    pub model_matrix: [[f32; 4]; 4],
    pub building_id: u32, // 255 = citizen marker
    pub tint_color: [f32; 4],
}

impl CitizenInstance {
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
            array_stride: std::mem::size_of::<CitizenInstance>() as wgpu::BufferAddress,
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
    road_render_pipeline: wgpu::RenderPipeline,
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
    
    // Погодные эффекты
    weather_buffer: wgpu::Buffer,
    weather_bind_group: wgpu::BindGroup,
    weather_uniform: WeatherUniform,
    weather_pipeline: wgpu::RenderPipeline,
    
    // Частицы зданий
    building_particles: Vec<BuildingParticle>,
    building_particle_buffer: wgpu::Buffer,
    building_particle_storage_buffer: wgpu::Buffer,
    building_particle_bind_group: wgpu::BindGroup,
    building_particle_uniform: BuildingParticleUniform,
    building_particle_pipeline: wgpu::RenderPipeline,
    
    // UI экранные униформы (отдельно от мировой камеры)
    screen_buffer: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
    screen_uniform: ScreenUniform,
    
    // Текстуры
    texture_bind_group: Option<wgpu::BindGroup>,
    road_texture_bind_group: Option<wgpu::BindGroup>,
    
    // Временные буферы для инстансов
    tile_instances: Vec<TileInstance>,
    building_instances: Vec<BuildingInstance>,
    citizen_instances: Vec<CitizenInstance>,
    road_instances: Vec<RoadInstance>,
    road_preview_instances: Vec<RoadInstance>,
    building_preview_instances: Vec<BuildingInstance>,
    ui_rects: Vec<UIRect>,
    minimap_instances: Vec<UIRect>,
    max_instances: usize,
    tile_instance_buffer: wgpu::Buffer,
    building_instance_buffer: wgpu::Buffer,
    citizen_instance_buffer: wgpu::Buffer,
    road_instance_buffer: wgpu::Buffer,
    road_preview_instance_buffer: wgpu::Buffer,
    building_preview_instance_buffer: wgpu::Buffer,
    ui_rect_buffer: wgpu::Buffer,
    minimap_buffer: wgpu::Buffer,
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
        
        // Загружаем спрайтшит тайлов
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
        
        // Загружаем текстуру деревьев
        let trees_bytes = include_bytes!("../assets/trees.png");
        let trees_image = image::load_from_memory(trees_bytes)?;
        let trees_rgba = trees_image.to_rgba8();
        
        let (trees_width, trees_height) = trees_rgba.dimensions();
        
        let trees_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Trees Texture"),
            size: wgpu::Extent3d {
                width: trees_width,
                height: trees_height,
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
                texture: &trees_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &trees_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * trees_width),
                rows_per_image: Some(trees_height),
            },
            wgpu::Extent3d {
                width: trees_width,
                height: trees_height,
                depth_or_array_layers: 1,
            },
        );
        
        let trees_view = trees_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let trees_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        // Загружаем текстуру зданий
        let buildings_bytes = include_bytes!("../assets/buildings.png");
        let buildings_image = image::load_from_memory(buildings_bytes)?;
        let buildings_rgba = buildings_image.to_rgba8();
        
        let (buildings_width, buildings_height) = buildings_rgba.dimensions();
        
        let buildings_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Buildings Texture"),
            size: wgpu::Extent3d {
                width: buildings_width,
                height: buildings_height,
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
                texture: &buildings_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &buildings_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * buildings_width),
                rows_per_image: Some(buildings_height),
            },
            wgpu::Extent3d {
                width: buildings_width,
                height: buildings_height,
                depth_or_array_layers: 1,
            },
        );
        
        let buildings_view = buildings_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let buildings_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
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
        
        // Создаём screen uniform для UI (экранные координаты)
        let screen_uniform = ScreenUniform {
            screen_size: [size.width as f32, size.height as f32],
            padding: [0.0, 0.0],
        };
        
        let screen_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Buffer"),
            contents: bytemuck::cast_slice(&[screen_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Создаём bind group layout для экрана (UI)
        let screen_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("screen_bind_group_layout"),
        });
        
        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
            label: Some("screen_bind_group"),
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
                // Текстура деревьев
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Текстура зданий
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&trees_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&trees_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&buildings_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&buildings_sampler),
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
        
        let citizen_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Citizen Instance Buffer"),
            size: (std::mem::size_of::<CitizenInstance>() * max_instances) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let road_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Road Instance Buffer"),
            size: (std::mem::size_of::<RoadInstance>() * max_instances) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let road_preview_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Road Preview Instance Buffer"),
            size: (std::mem::size_of::<RoadInstance>() * max_instances) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let building_preview_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Building Preview Instance Buffer"),
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
        
        let minimap_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Minimap Buffer"),
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
                entry_point: "vs_main_building",
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
        
        // Создаём render pipeline для дорог
        let road_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Road Render Pipeline"),
            layout: Some(&building_render_pipeline_layout), // используем тот же layout
            vertex: wgpu::VertexState {
                module: &building_shader,
                entry_point: "vs_main_road",
                buffers: &[Vertex::desc(), RoadInstance::desc()],
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
        
        // UI Rect пайплайн (с экранными координатами, без текстур)
        let ui_rect_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Rect Render Pipeline Layout"),
            bind_group_layouts: &[&screen_bind_group_layout],
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
                cull_mode: None,  // Отключаем culling для UI - плоские прямоугольники
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
        
        // Создаем погодные эффекты
        let weather_uniform = WeatherUniform::new();
        
        let weather_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Weather Buffer"),
            contents: bytemuck::cast_slice(&[weather_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let weather_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("weather_bind_group_layout"),
        });
        
        let weather_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &weather_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: weather_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: screen_buffer.as_entire_binding(),
                },
            ],
            label: Some("weather_bind_group"),
        });
        
        // Загружаем шейдер погоды
        let weather_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Weather Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/weather.wgsl").into()),
        });
        
        let weather_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Weather Pipeline Layout"),
            bind_group_layouts: &[&weather_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let weather_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Weather Pipeline"),
            layout: Some(&weather_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &weather_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &weather_shader,
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
        
        // Инициализация частиц зданий
        let building_particle_uniform = BuildingParticleUniform::new();
        let building_particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Building Particle Buffer"),
            size: std::mem::size_of::<BuildingParticleUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let building_particle_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Building Particle Storage Buffer"),
            size: (std::mem::size_of::<BuildingParticle>() * 1000) as u64, // Максимум 1000 частиц
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let building_particle_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("building_particle_bind_group_layout"),
        });
        
        let building_particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &building_particle_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: building_particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: building_particle_storage_buffer.as_entire_binding(),
                },
            ],
            label: Some("building_particle_bind_group"),
        });
        
        // Загружаем шейдер частиц зданий
        let building_particle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Building Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/building_particles.wgsl").into()),
        });
        
        let building_particle_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Building Particle Pipeline Layout"),
            bind_group_layouts: &[&building_particle_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let building_particle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Building Particle Pipeline"),
            layout: Some(&building_particle_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &building_particle_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &building_particle_shader,
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
            road_render_pipeline,
            ui_render_pipeline,
            ui_rect_render_pipeline,
            tile_vertex_buffer,
            tile_index_buffer,
            building_vertex_buffer,
            building_index_buffer,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            weather_buffer,
            weather_bind_group,
            weather_uniform,
            weather_pipeline,
            building_particles: Vec::new(),
            building_particle_buffer,
            building_particle_storage_buffer,
            building_particle_bind_group,
            building_particle_uniform,
            building_particle_pipeline,
            screen_buffer,
            screen_bind_group,
            screen_uniform,
            texture_bind_group: Some(texture_bind_group),
            road_texture_bind_group: None,
            tile_instances: Vec::new(),
            building_instances: Vec::new(),
            citizen_instances: Vec::new(),
            road_instances: Vec::new(),
            road_preview_instances: Vec::new(),
            building_preview_instances: Vec::new(),
            ui_rects: Vec::new(),
            minimap_instances: Vec::new(),
            max_instances,
            tile_instance_buffer,
            building_instance_buffer,
            citizen_instance_buffer,
            road_instance_buffer,
            road_preview_instance_buffer,
            building_preview_instance_buffer,
            ui_rect_buffer,
            minimap_buffer,
        })
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            // Обновляем screen_uniform для UI при изменении размера экрана
            self.screen_uniform.screen_size = [new_size.width as f32, new_size.height as f32];
            self.queue.write_buffer(&self.screen_buffer, 0, bytemuck::cast_slice(&[self.screen_uniform]));
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
    
    // Обновление погодных эффектов
    pub fn update_weather(&mut self, weather: WeatherKind, time: f32, intensity: f32) {
        self.weather_uniform.weather_type = match weather {
            WeatherKind::Clear => 0,
            WeatherKind::Rain => 1,
            WeatherKind::Fog => 2,
            WeatherKind::Snow => 3,
        };
        self.weather_uniform.time = time;
        self.weather_uniform.intensity = intensity;
        
        self.queue.write_buffer(
            &self.weather_buffer,
            0,
            bytemuck::cast_slice(&[self.weather_uniform])
        );
    }
    
    // Обновление частиц зданий
    pub fn update_building_particles(&mut self, buildings: &[crate::types::Building], time: f32) {
        self.building_particles.clear();
        
        for building in buildings {
            match building.kind {
                BuildingKind::Smelter => {
                    // Искры от плавильни
                    if building.timer_ms > 0 {
                        for _ in 0..3 {
                            let angle = (time * 2.0 + building.pos.x as f32 + building.pos.y as f32) % (std::f32::consts::TAU);
                            let speed = 0.5 + (time * 0.1) % 0.3;
                            
                            self.building_particles.push(BuildingParticle {
                                position: [building.pos.x as f32 * 32.0 + 16.0, building.pos.y as f32 * 32.0 + 16.0],
                                velocity: [angle.cos() * speed, angle.sin() * speed - 0.2],
                                life: 1.0,
                                size: 2.0 + (time * 0.1) % 1.0,
                                color: [1.0, 0.8, 0.2, 0.8], // Оранжевые искры
                            });
                        }
                    }
                }
                BuildingKind::Kiln | BuildingKind::Bakery => {
                    // Дым от печи/пекарни
                    if building.timer_ms > 0 {
                        for _ in 0..2 {
                            let offset_x = (time * 0.5 + building.pos.x as f32) % 1.0 - 0.5;
                            let offset_y = (time * 0.3 + building.pos.y as f32) % 1.0 - 0.5;
                            
                            self.building_particles.push(BuildingParticle {
                                position: [building.pos.x as f32 * 32.0 + 16.0 + offset_x * 8.0, building.pos.y as f32 * 32.0 + 16.0 + offset_y * 8.0],
                                velocity: [0.0, -0.3],
                                life: 1.0,
                                size: 3.0 + (time * 0.1) % 2.0,
                                color: [0.6, 0.6, 0.6, 0.6], // Серый дым
                            });
                        }
                    }
                }
                BuildingKind::Mill => {
                    // Пыль от мельницы
                    if building.timer_ms > 0 {
                        for _ in 0..2 {
                            let angle = (time * 3.0 + building.pos.x as f32 + building.pos.y as f32) % (std::f32::consts::TAU);
                            let radius = 8.0 + (time * 0.2) % 4.0;
                            
                            self.building_particles.push(BuildingParticle {
                                position: [building.pos.x as f32 * 32.0 + 16.0 + angle.cos() * radius, building.pos.y as f32 * 32.0 + 16.0 + angle.sin() * radius],
                                velocity: [angle.cos() * 0.1, angle.sin() * 0.1],
                                life: 1.0,
                                size: 1.5 + (time * 0.1) % 1.0,
                                color: [0.9, 0.9, 0.8, 0.7], // Бежевая пыль
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Обновляем униформы
        self.building_particle_uniform.time = time;
        self.building_particle_uniform.particle_count = self.building_particles.len() as u32;
        
        // Записываем данные в буферы
        self.queue.write_buffer(
            &self.building_particle_buffer,
            0,
            bytemuck::cast_slice(&[self.building_particle_uniform])
        );
        
        if !self.building_particles.is_empty() {
            self.queue.write_buffer(
                &self.building_particle_storage_buffer,
                0,
                bytemuck::cast_slice(&self.building_particles)
            );
        }
    }
    
    // Подготовка тайлов для рендеринга (пиксельные координаты как в CPU)
    pub fn prepare_tiles(&mut self, world: &mut World, atlas: &crate::atlas::TileAtlas, min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32, hovered_tile: Option<glam::IVec2>, show_deposits: bool) {
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
                
                // Проверяем наличие депозитов ресурсов
                let pos = glam::IVec2::new(mx, my);
                let has_clay = world.has_clay_deposit(pos);
                let has_stone = world.has_stone_deposit(pos);
                let has_iron = world.has_iron_deposit(pos);
                
                let (tile_id, tint) = if show_deposits && (has_clay || has_stone || has_iron) {
                    // Депозит ресурса - используем тайл (6, 5) из spritesheet.png
                    let deposit_tile_id = 61; // тайл (6, 1) - попробуем другой
                    
                    // Определяем цвет депозита (приоритет: железо > камень > глина)
                    let deposit_tint = if has_iron {
                        [0.3, 0.3, 0.3, 1.0] // темно-серый для железа
                    } else if has_stone {
                        [0.6, 0.6, 0.6, 1.0] // серый для камня
                    } else {
                        [0.8, 0.6, 0.4, 1.0] // коричневый для глины
                    };
                    
                    (deposit_tile_id, deposit_tint)
                } else {
                    // Обычный тайл
                    let tile_id = match kind {
                        TileKind::Grass => 22,
                        TileKind::Forest => 40, 
                        TileKind::Water => 110,
                    };
                    
                    // Определяем биом для тинтинга
                    let biome = world.biome(glam::IVec2::new(mx, my));
                    
                    // Применяем тинт биома (более яркий)
                    let biome_tint = match (kind, biome) {
                        (TileKind::Grass, BiomeKind::Swamp) => [0.4, 0.3, 0.2, 1.0],   // темный коричневый оттенок
                        (TileKind::Grass, BiomeKind::Rocky) => [0.8, 0.8, 0.8, 1.0],   // светлый серый оттенок
                        (TileKind::Forest, BiomeKind::Swamp) => [0.4, 0.3, 0.2, 1.0],  // темный коричневый оттенок для леса
                        (TileKind::Forest, BiomeKind::Rocky) => [0.8, 0.8, 0.8, 1.0],  // светлый серый оттенок для леса
                        _ => [1.0, 1.0, 1.0, 1.0], // без тинтинга для лугов и воды
                    };
                    
                    // Подсветка при наведении - желтый тинт поверх биомного
                    let tint = if hovered_tile == Some(glam::IVec2::new(mx, my)) {
                        [
                            biome_tint[0] * 1.3,
                            biome_tint[1] * 1.3, 
                            biome_tint[2] * 0.7,
                            biome_tint[3]
                        ]
                    } else {
                        biome_tint
                    };
                    
                    (tile_id, tint)
                };
                
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
    
    // Применяет полноэкранный tint (для погоды и ночного освещения)
    pub fn apply_screen_tint(&mut self, color: [f32; 4]) {
        let w = self.size.width as f32;
        let h = self.size.height as f32;
        self.add_ui_rect(0.0, 0.0, w, h, color);
    }
    
    // Bitmap шрифт 3x5 для цифр и букв
    fn get_glyph_pattern(ch: u8) -> [u8; 15] {
        match ch.to_ascii_uppercase() {
            b'0' => [1,1,1, 1,0,1, 1,0,1, 1,0,1, 1,1,1],
            b'1' => [0,1,0, 1,1,0, 0,1,0, 0,1,0, 1,1,1],
            b'2' => [1,1,1, 0,0,1, 1,1,1, 1,0,0, 1,1,1],
            b'3' => [1,1,1, 0,0,1, 0,1,1, 0,0,1, 1,1,1],
            b'4' => [1,0,1, 1,0,1, 1,1,1, 0,0,1, 0,0,1],
            b'5' => [1,1,1, 1,0,0, 1,1,1, 0,0,1, 1,1,1],
            b'6' => [1,1,1, 1,0,0, 1,1,1, 1,0,1, 1,1,1],
            b'7' => [1,1,1, 0,0,1, 0,1,0, 0,1,0, 0,1,0],
            b'8' => [1,1,1, 1,0,1, 1,1,1, 1,0,1, 1,1,1],
            b'9' => [1,1,1, 1,0,1, 1,1,1, 0,0,1, 1,1,1],
            b'A' => [0,1,0, 1,0,1, 1,1,1, 1,0,1, 1,0,1],
            b'B' => [1,1,0, 1,0,1, 1,1,0, 1,0,1, 1,1,0],
            b'C' => [0,1,1, 1,0,0, 1,0,0, 1,0,0, 0,1,1],
            b'D' => [1,1,0, 1,0,1, 1,0,1, 1,0,1, 1,1,0],
            b'E' => [1,1,1, 1,0,0, 1,1,0, 1,0,0, 1,1,1],
            b'F' => [1,1,1, 1,0,0, 1,1,0, 1,0,0, 1,0,0],
            b'G' => [0,1,1, 1,0,0, 1,0,1, 1,0,1, 0,1,1],
            b'H' => [1,0,1, 1,0,1, 1,1,1, 1,0,1, 1,0,1],
            b'I' => [1,1,1, 0,1,0, 0,1,0, 0,1,0, 1,1,1],
            b'J' => [0,0,1, 0,0,1, 0,0,1, 1,0,1, 0,1,0],
            b'K' => [1,0,1, 1,1,0, 1,0,0, 1,1,0, 1,0,1],
            b'L' => [1,0,0, 1,0,0, 1,0,0, 1,0,0, 1,1,1],
            b'M' => [1,0,1, 1,1,1, 1,0,1, 1,0,1, 1,0,1],
            b'N' => [1,0,1, 1,1,1, 1,1,1, 1,0,1, 1,0,1],
            b'O' => [0,1,0, 1,0,1, 1,0,1, 1,0,1, 0,1,0],
            b'P' => [1,1,0, 1,0,1, 1,1,0, 1,0,0, 1,0,0],
            b'Q' => [0,1,0, 1,0,1, 1,0,1, 1,1,0, 0,1,1],
            b'R' => [1,1,0, 1,0,1, 1,1,0, 1,0,1, 1,0,1],
            b'S' => [0,1,1, 1,0,0, 0,1,0, 0,0,1, 1,1,0],
            b'T' => [1,1,1, 0,1,0, 0,1,0, 0,1,0, 0,1,0],
            b'U' => [1,0,1, 1,0,1, 1,0,1, 1,0,1, 0,1,0],
            b'V' => [1,0,1, 1,0,1, 1,0,1, 1,0,1, 0,1,0],
            b'W' => [1,0,1, 1,0,1, 1,0,1, 1,1,1, 1,0,1],
            b'X' => [1,0,1, 1,0,1, 0,1,0, 1,0,1, 1,0,1],
            b'Y' => [1,0,1, 1,0,1, 0,1,0, 0,1,0, 0,1,0],
            b'Z' => [1,1,1, 0,0,1, 0,1,0, 1,0,0, 1,1,1],
            _ => [0,0,0, 0,0,0, 0,0,0, 0,0,0, 0,0,0],
        }
    }
    
    // Рисует один глиф bitmap шрифта 3x5
    fn draw_glyph(&mut self, x: f32, y: f32, ch: u8, color: [f32; 4], scale: f32) {
        let pattern = Self::get_glyph_pattern(ch);
        let px = 2.0 * scale; // размер одного пикселя глифа
        
        for row in 0..5 {
            for col in 0..3 {
                if pattern[row * 3 + col] == 1 {
                    let gx = x + col as f32 * px;
                    let gy = y + row as f32 * px;
                    self.add_ui_rect(gx, gy, px, px, color);
                }
            }
        }
    }
    
    // Рисует число (как draw_number в CPU версии)
    pub fn draw_number(&mut self, x: f32, y: f32, mut n: u32, color: [f32; 4], scale: f32) {
        let mut digits: [u8; 12] = [0; 12];
        let mut len = 0;
        
        if n == 0 {
            digits[0] = b'0';
            len = 1;
        } else {
            while n > 0 && len < digits.len() {
                let d = (n % 10) as u8;
                n /= 10;
                digits[len] = b'0' + d;
                len += 1;
            }
        }
        
        let px = 2.0 * scale;
        let char_width = 4.0 * px; // ширина символа с отступом
        let mut current_x = x;
        
        for i in (0..len).rev() {
            self.draw_glyph(current_x, y, digits[i], color, scale);
            current_x += char_width;
        }
    }
    
    // Рисует текст
    pub fn draw_text(&mut self, x: f32, y: f32, text: &[u8], color: [f32; 4], scale: f32) {
        let px = 2.0 * scale;
        let char_width = 4.0 * px;
        let mut current_x = x;
        
        for &ch in text {
            if ch == b' ' {
                current_x += char_width;
                continue;
            }
            self.draw_glyph(current_x, y, ch, color, scale);
            current_x += char_width;
        }
    }
    
    // Рисует кнопку (прямоугольник с текстом)
    pub fn draw_button(&mut self, x: f32, y: f32, w: f32, h: f32, text: &[u8], active: bool, scale: f32) {
        // Цвета кнопки
        let bg_color = if active { 
            [185.0/255.0, 140.0/255.0, 95.0/255.0, 220.0/255.0] 
        } else { 
            [140.0/255.0, 105.0/255.0, 75.0/255.0, 180.0/255.0] 
        };
        
        // Фон кнопки
        self.add_ui_rect(x, y, w, h, bg_color);
        
        // Верхний блик
        let band = (2.0 * scale).max(2.0);
        self.add_ui_rect(x, y, w, band, [1.0, 1.0, 1.0, 0.27]);
        
        // Нижняя тень
        self.add_ui_rect(x, y + h - band, w, band, [0.0, 0.0, 0.0, 0.23]);
        
        // Текст по центру
        let px = 2.0 * scale;
        let text_w = text.len() as f32 * 4.0 * px;
        let text_h = 5.0 * px;
        let text_x = x + (w - text_w) / 2.0;
        let text_y = y + (h - text_h) / 2.0;
        
        self.draw_text(text_x, text_y, text, [220.0/255.0, 220.0/255.0, 220.0/255.0, 1.0], scale);
    }

    // Подготовка структур (здания и деревья) для рендеринга с правильной сортировкой
    pub fn prepare_structures(
        &mut self, 
        world: &mut crate::world::World, 
        buildings: &Vec<crate::types::Building>,
        building_atlas: &Option<crate::atlas::BuildingAtlas>,
        tree_atlas: &Option<crate::atlas::TreeAtlas>,
        tile_atlas: &crate::atlas::TileAtlas,
        min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
        highlighted_building: Option<glam::IVec2>, // Позиция выделенного здания
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
        
        // Размер зданий - масштабируем до размера тайла, сохраняя пропорции
        let (building_width, building_height) = if let Some(building_atlas) = building_atlas {
            let original_w = building_atlas.w as f32;
            let original_h = building_atlas.h as f32;
            
            // Масштабируем до размера тайла, сохраняя пропорции
            let tile_size = tile_w_px as f32;
            let scale = tile_size / original_w.max(original_h);
            (original_w * scale, original_h * scale * 0.5)
        } else {
            let tile_size = tile_w_px as f32;
            (tile_size, tile_size)
        };
        
        // Размер деревьев - масштабируем до размера тайла, сохраняя пропорции
        let tile_size = tile_w_px as f32;
        let (tree_width, tree_height) = if let Some(tree_atlas) = tree_atlas {
            let original_w = tree_atlas.w as f32;
            let original_h = tree_atlas.h as f32;
            
            // Масштабируем до размера тайла, сохраняя пропорции
            let scale = tile_size / original_w.max(original_h);
            (original_w * scale, original_h * scale * 0.6)
        } else {
            (tile_size, tile_size)
        };
        
        for s in min_s..=max_s {
            for mx in min_tx..=max_tx {
                let my = s - mx;
                if my < min_ty || my > max_ty { continue; }
                
                // Сначала деревья (рисуем раньше для правильного порядка)
                if world.has_tree(IVec2::new(mx, my)) {
                    let stage = world.tree_stage(IVec2::new(mx, my)).unwrap_or(2) as u32;
                    
                    // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ (как в CPU версии)
                    let iso_x = (mx - my) as f32 * half_w;
                    let iso_y = (mx + my) as f32 * half_h;
                    
                    // Смещение дерева вверх (немного меньше чем здания)
                    let tree_off = half_h * 3.8; 
                    // let tree_off = 46.0; 
                    let final_y = iso_y - tree_off;
                    
                    // Матрица трансформации дерева (с правильными пропорциями)
                    let transform = Mat4::from_scale_rotation_translation(
                        glam::Vec3::new(tree_width, tree_height, 1.0),
                        glam::Quat::IDENTITY,
                        glam::Vec3::new(iso_x, -final_y, 0.0)
                    );
                    
                    // Используем ID 100 + stage для деревьев
                    let tree_id = 100 + stage;
                    
                    let instance = BuildingInstance {
                        model_matrix: transform.to_cols_array_2d(),
                        building_id: tree_id,
                        tint_color: [1.0, 1.0, 1.0, 1.0],
                        padding: [0; 3],
                    };
                    self.building_instances.push(instance);
                }
                
                // Затем здания
                if let Some(&building_idx) = buildings_by_pos.get(&(mx, my)) {
                    let building = &buildings[building_idx];
                    
                    // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ (как в CPU версии)
                    let iso_x = (mx - my) as f32 * half_w;
                    let iso_y = (mx + my) as f32 * half_h;
                    
                    // Смещение здания вверх (на тайле)
                    let building_off = half_h * 2.0; 
                    let final_y = iso_y - building_off; // вычитаем, чтобы поднять здание
                    
                    // Матрица трансформации здания (с правильными пропорциями)
                    let transform = Mat4::from_scale_rotation_translation(
                        glam::Vec3::new(building_width, building_height, 1.0),
                        glam::Quat::IDENTITY,
                        glam::Vec3::new(iso_x, -final_y, 0.0) // используем -final_y как у деревьев
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
                    
                    // Подсветка здания при наведении
                    let is_highlighted = highlighted_building.map_or(false, |pos| pos.x == mx && pos.y == my);
                    let tint_color = if is_highlighted {
                        [1.3, 1.3, 1.0, 1.0] // Желтоватое свечение при наведении
                    } else {
                        [1.0, 1.0, 1.0, 1.0] // Белый цвет по умолчанию
                    };
                    
                    let instance = BuildingInstance {
                        model_matrix: transform.to_cols_array_2d(),
                        building_id,
                        tint_color,
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
    
    pub fn prepare_citizens(
        &mut self,
        citizens: &Vec<crate::types::Citizen>,
        buildings: &Vec<crate::types::Building>,
        tile_atlas: &crate::atlas::TileAtlas,
    ) {
        use crate::palette::building_color;
        use glam::{Mat4, Vec3};
        
        self.citizen_instances.clear();
        
        let half_w = tile_atlas.half_w as f32;
        let half_h = tile_atlas.half_h as f32;
        let citizen_size = (half_w * 0.7).max(12.0); // увеличенный размер маркера для видимости
        
        for c in citizens.iter() {
            // Интерполяция позиции для движущихся граждан (как в CPU версии)
            let (fx, fy) = if c.moving {
                let dx = (c.target.x - c.pos.x) as f32;
                let dy = (c.target.y - c.pos.y) as f32;
                (c.pos.x as f32 + dx * c.progress, c.pos.y as f32 + dy * c.progress)
            } else {
                (c.pos.x as f32, c.pos.y as f32)
            };
            
            // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ (аналогично зданиям)
            let iso_x = (fx - fy) * half_w;
            let iso_y = (fx + fy) * half_h;
            
            // Смещение вверх от базы тайла (как в CPU: base_y - half_h/3)
            let y_offset = -half_h / 3.0;
            
            // Цвет гражданина зависит от места работы
            let mut col = [255.0/255.0, 230.0/255.0, 120.0/255.0, 1.0]; // желтоватый по умолчанию
            if let Some(wp) = c.workplace {
                if let Some(b) = buildings.iter().find(|b| b.pos == wp) {
                    let bcol = building_color(b.kind);
                    col = [
                        bcol[0] as f32 / 255.0,
                        bcol[1] as f32 / 255.0,
                        bcol[2] as f32 / 255.0,
                        1.0,
                    ];
                }
            }
            
            // Матрица трансформации для гражданина
            let transform = Mat4::from_scale_rotation_translation(
                Vec3::new(citizen_size, citizen_size, 1.0),
                glam::Quat::IDENTITY,
                Vec3::new(iso_x, -(iso_y + y_offset), 0.0), // минус как у тайлов и зданий!
            );
            
            let instance = CitizenInstance {
                model_matrix: transform.to_cols_array_2d(),
                building_id: 255, // специальный ID для граждан
                tint_color: col,
            };
            
            self.citizen_instances.push(instance);
        }
        
        // Обновляем буфер инстансов граждан
        if !self.citizen_instances.is_empty() {
            self.queue.write_buffer(
                &self.citizen_instance_buffer,
                0,
                bytemuck::cast_slice(&self.citizen_instances)
            );
        }
    }
    
    pub fn prepare_roads(
        &mut self,
        world: &mut crate::world::World,
        road_atlas: &crate::atlas::RoadAtlas,
        tile_atlas: &crate::atlas::TileAtlas,
        min_tx: i32, min_ty: i32, max_tx: i32, max_ty: i32,
    ) {
        use glam::{Mat4, Vec3};
        
        self.road_instances.clear();
        
        let half_w = tile_atlas.half_w as f32;
        let half_h = tile_atlas.half_h as f32;
        
        // Проходим по всем видимым тайлам и ищем дороги
        for mx in min_tx..=max_tx {
            for my in min_ty..=max_ty {
                if !world.is_road(glam::IVec2::new(mx, my)) { continue; }
                
                // Вычисляем маску соединений дороги (как в CPU версии)
                let mut mask = 0u8;
                if world.is_road(glam::IVec2::new(mx, my - 1)) { mask |= 0b0001; } // север
                if world.is_road(glam::IVec2::new(mx + 1, my)) { mask |= 0b0010; } // восток  
                if world.is_road(glam::IVec2::new(mx, my + 1)) { mask |= 0b0100; } // юг
                if world.is_road(glam::IVec2::new(mx - 1, my)) { mask |= 0b1000; } // запад
                
                // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ (как у зданий)
                let iso_x = (mx - my) as f32 * half_w;
                let iso_y = ((mx + my) as f32 * half_h) - half_h * 0.5;
                
                // Матрица трансформации дороги (размер как у тайла)
                let transform = Mat4::from_scale_rotation_translation(
                    Vec3::new(half_w * 2.0, half_h * 2.0, 1.0),
                    glam::Quat::IDENTITY,
                    Vec3::new(iso_x, -iso_y, 0.0), // минус как у зданий
                );
                
                let instance = RoadInstance {
                    model_matrix: transform.to_cols_array_2d(),
                    road_mask: mask as u32,
                    tint_color: [1.0, 1.0, 1.0, 1.0], // белый цвет по умолчанию
                    padding: [0; 3],
                };
                
                self.road_instances.push(instance);
            }
        }
        
        // Обновляем буфер инстансов дорог
        if !self.road_instances.is_empty() {
            self.queue.write_buffer(
                &self.road_instance_buffer,
                0,
                bytemuck::cast_slice(&self.road_instances)
            );
        }
    }
    
    pub fn clear_road_preview(&mut self) {
        self.road_preview_instances.clear();
    }
    
    pub fn prepare_building_preview(
        &mut self,
        building_kind: crate::types::BuildingKind,
        tile_pos: glam::IVec2,
        is_allowed: bool,
        building_atlas: &Option<crate::atlas::BuildingAtlas>,
        tile_atlas: &crate::atlas::TileAtlas,
    ) {
        use crate::types::BuildingKind;
        use glam::{Mat4, Vec3};
        
        self.building_preview_instances.clear();
        
        if building_atlas.is_none() { return; }
        
        let half_w = tile_atlas.half_w as f32;
        let half_h = tile_atlas.half_h as f32;
        let tile_w_px = tile_atlas.half_w * 2 + 1;
        
        // Размер зданий - масштабируем до размера тайла, сохраняя пропорции (как в prepare_structures)
        let (building_width, building_height) = if let Some(building_atlas) = building_atlas {
            let original_w = building_atlas.w as f32;
            let original_h = building_atlas.h as f32;
            
            // Масштабируем до размера тайла, сохраняя пропорции
            let tile_size = tile_w_px as f32;
            let scale = tile_size / original_w.max(original_h);
            (original_w * scale, original_h * scale * 0.5)
        } else {
            let tile_size = tile_w_px as f32;
            (tile_size, tile_size)
        };
        
        // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ
        let iso_x = (tile_pos.x - tile_pos.y) as f32 * half_w;
        let iso_y = (tile_pos.x + tile_pos.y) as f32 * half_h;
        
        // Смещение здания вверх (на тайле)
        let building_off = half_h * 2.0;
        let final_y = iso_y - building_off;
        
        // Матрица трансформации здания
        let transform = Mat4::from_scale_rotation_translation(
            Vec3::new(building_width, building_height, 1.0),
            glam::Quat::IDENTITY,
            Vec3::new(iso_x, -final_y, 0.0)
        );
        
        // Конвертируем BuildingKind в u32 ID
        let building_id = match building_kind {
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
        
        // Цвет предпросмотра: зеленоватый если можно построить, красноватый если нельзя
        let tint_color = if is_allowed {
            [0.5, 1.0, 0.5, 0.6] // Зеленоватый полупрозрачный
        } else {
            [1.0, 0.5, 0.5, 0.6] // Красноватый полупрозрачный
        };
        
        let instance = BuildingInstance {
            model_matrix: transform.to_cols_array_2d(),
            building_id,
            tint_color,
            padding: [0; 3],
        };
        
        self.building_preview_instances.push(instance);
        
        // Обновляем буфер инстансов предпросмотра зданий
        if !self.building_preview_instances.is_empty() {
            self.queue.write_buffer(
                &self.building_preview_instance_buffer,
                0,
                bytemuck::cast_slice(&self.building_preview_instances)
            );
        }
    }
    
    pub fn clear_building_preview(&mut self) {
        self.building_preview_instances.clear();
    }
    
    pub fn prepare_minimap(
        &mut self,
        world: &mut crate::world::World,
        buildings: &[crate::types::Building],
        cam_x: f32,
        cam_y: f32,
        minimap_x: i32,
        minimap_y: i32,
        minimap_w: i32,
        minimap_h: i32,
        cell_size: i32,
    ) {
        use crate::types::{TileKind, BiomeKind};
        
        self.minimap_instances.clear();
        
        // Границы миникарты в тайлах (динамические вокруг камеры)
        let cam_tile_x = (cam_x / 64.0) as i32; // преобразуем координаты камеры в тайлы
        let cam_tile_y = (cam_y / 32.0) as i32;
        
        // Размер области миникарты в тайлах (соответствует размеру виджета)
        let map_radius = 30; // радиус области вокруг камеры
        
        let min_tx = cam_tile_x - map_radius;
        let min_ty = cam_tile_y - map_radius;
        let max_tx = cam_tile_x + map_radius;
        let max_ty = cam_tile_y + map_radius;
        
        // Добавляем тестовый квадрат в левый верхний угол миникарты
        let test_transform = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(20.0, 20.0, 1.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(minimap_x as f32 + 10.0, minimap_y as f32 + 10.0, 0.0),
        );
        
        self.minimap_instances.push(UIRect {
            model_matrix: test_transform.to_cols_array_2d(),
            color: [1.0, 0.0, 0.0, 1.0], // ярко-красный тестовый квадрат
        });
        
        // Рендерим тайлы миникарты с тинтами биомов
        for tx in min_tx..=max_tx {
            for ty in min_ty..=max_ty {
                let tile_kind = world.get_tile(tx, ty);
                let biome = world.biome(glam::IVec2::new(tx, ty));
                
                // Базовые цвета тайлов
                let base_color = match tile_kind {
                    TileKind::Grass => [0.2, 0.6, 0.2, 1.0], // зеленый
                    TileKind::Forest => [0.1, 0.4, 0.1, 1.0], // темно-зеленый
                    TileKind::Water => [0.2, 0.4, 0.8, 1.0], // синий
                };
                
                // Применяем тинт биома (те же значения, что и в основном рендерере)
                let biome_tint = match (tile_kind, biome) {
                    (TileKind::Grass, BiomeKind::Swamp) => [0.4, 0.3, 0.2, 1.0],   // темный коричневый оттенок
                    (TileKind::Grass, BiomeKind::Rocky) => [0.8, 0.8, 0.8, 1.0],   // светлый серый оттенок
                    (TileKind::Forest, BiomeKind::Swamp) => [0.4, 0.3, 0.2, 1.0],  // темный коричневый оттенок для леса
                    (TileKind::Forest, BiomeKind::Rocky) => [0.8, 0.8, 0.8, 1.0],  // светлый серый оттенок для леса
                    _ => [1.0, 1.0, 1.0, 1.0], // без тинтинга для лугов и воды
                };
                
                // Применяем тинт к базовому цвету
                let color = [
                    base_color[0] * biome_tint[0],
                    base_color[1] * biome_tint[1], 
                    base_color[2] * biome_tint[2],
                    base_color[3] * biome_tint[3]
                ];
                
                let x = minimap_x + (tx - min_tx) * cell_size;
                let y = minimap_y + (ty - min_ty) * cell_size;
                
                // Проверяем, что координаты в пределах миникарты
                if x >= minimap_x && x < minimap_x + minimap_w && 
                   y >= minimap_y && y < minimap_y + minimap_h {
                    
                    let transform = glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::new(cell_size as f32, cell_size as f32, 1.0),
                        glam::Quat::IDENTITY,
                        glam::Vec3::new(x as f32, y as f32, 0.0),
                    );
                    
                    self.minimap_instances.push(UIRect {
                        model_matrix: transform.to_cols_array_2d(),
                        color,
                    });
                }
            }
        }
        
        // Рендерим депозиты ресурсов на миникарте
        for tx in min_tx..=max_tx {
            for ty in min_ty..=max_ty {
                let pos = glam::IVec2::new(tx, ty);
                
                // Проверяем наличие депозитов
                let has_clay = world.has_clay_deposit(pos);
                let has_stone = world.has_stone_deposit(pos);
                let has_iron = world.has_iron_deposit(pos);
                
                if has_clay || has_stone || has_iron {
                    let x = minimap_x + (tx - min_tx) * cell_size;
                    let y = minimap_y + (ty - min_ty) * cell_size;
                    
                    // Проверяем, что координаты в пределах миникарты
                    if x >= minimap_x && x < minimap_x + minimap_w && 
                       y >= minimap_y && y < minimap_y + minimap_h {
                        
                        // Определяем цвет депозита (приоритет: железо > камень > глина)
                        let deposit_color = if has_iron {
                            [0.3, 0.3, 0.3, 0.8] // темно-серый для железа
                        } else if has_stone {
                            [0.6, 0.6, 0.6, 0.8] // серый для камня
                        } else {
                            [0.8, 0.6, 0.4, 0.8] // коричневый для глины
                        };
                        
                        let transform = glam::Mat4::from_scale_rotation_translation(
                            glam::Vec3::new(cell_size as f32 * 0.6, cell_size as f32 * 0.6, 1.0),
                            glam::Quat::IDENTITY,
                            glam::Vec3::new(x as f32, y as f32, 0.0),
                        );
                        
                        self.minimap_instances.push(UIRect {
                            model_matrix: transform.to_cols_array_2d(),
                            color: deposit_color,
                        });
                    }
                }
            }
        }
        
        // Рендерим здания на миникарте
        for building in buildings {
            // Проверяем, что здание в области миникарты
            if building.pos.x < min_tx || building.pos.x > max_tx ||
               building.pos.y < min_ty || building.pos.y > max_ty {
                continue;
            }
            
            // Преобразуем координаты здания в координаты миникарты
            let map_x = minimap_x + (building.pos.x - min_tx) * cell_size;
            let map_y = minimap_y + (building.pos.y - min_ty) * cell_size;
            
            if map_x >= minimap_x && map_x < minimap_x + minimap_w &&
               map_y >= minimap_y && map_y < minimap_y + minimap_h {
                
                let building_color = match building.kind {
                    crate::types::BuildingKind::House => [0.8, 0.6, 0.4, 1.0], // коричневый
                    crate::types::BuildingKind::Warehouse => [0.6, 0.4, 0.2, 1.0], // темно-коричневый
                    crate::types::BuildingKind::Lumberjack => [0.4, 0.2, 0.1, 1.0], // очень темно-коричневый
                    crate::types::BuildingKind::Forester => [0.3, 0.5, 0.2, 1.0], // зеленый
                    crate::types::BuildingKind::StoneQuarry => [0.5, 0.5, 0.5, 1.0], // серый
                    crate::types::BuildingKind::ClayPit => [0.6, 0.4, 0.2, 1.0], // коричневый
                    crate::types::BuildingKind::Kiln => [0.7, 0.5, 0.3, 1.0], // светло-коричневый
                    crate::types::BuildingKind::IronMine => [0.3, 0.3, 0.3, 1.0], // темно-серый
                    crate::types::BuildingKind::WheatField => [0.8, 0.8, 0.2, 1.0], // желтый
                    crate::types::BuildingKind::Mill => [0.7, 0.7, 0.7, 1.0], // светло-серый
                    crate::types::BuildingKind::Bakery => [0.9, 0.7, 0.4, 1.0], // светло-коричневый
                    crate::types::BuildingKind::Smelter => [0.4, 0.4, 0.6, 1.0], // сине-серый
                    crate::types::BuildingKind::Fishery => [0.2, 0.6, 0.8, 1.0], // голубой
                };
                
                let transform = glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::new((cell_size / 2) as f32, (cell_size / 2) as f32, 1.0),
                    glam::Quat::IDENTITY,
                    glam::Vec3::new(map_x as f32, map_y as f32, 0.0),
                );
                
                self.minimap_instances.push(UIRect {
                    model_matrix: transform.to_cols_array_2d(),
                    color: building_color,
                });
            }
        }
        
        // Рендерим дороги на миникарте
        for tx in min_tx..=max_tx {
            for ty in min_ty..=max_ty {
                if !world.is_road(glam::IVec2::new(tx, ty)) { continue; }
                
                // Преобразуем координаты дороги в координаты миникарты
                let map_x = minimap_x + (tx - min_tx) * cell_size;
                let map_y = minimap_y + (ty - min_ty) * cell_size;
                
                // Проверяем, что дорога в пределах миникарты
                if map_x >= minimap_x && map_x < minimap_x + minimap_w &&
                   map_y >= minimap_y && map_y < minimap_y + minimap_h {
                    
                    let transform = glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::new(cell_size as f32, cell_size as f32, 1.0),
                        glam::Quat::IDENTITY,
                        glam::Vec3::new(map_x as f32, map_y as f32, 0.0),
                    );
                    
                    self.minimap_instances.push(UIRect {
                        model_matrix: transform.to_cols_array_2d(),
                        color: [0.47, 0.43, 0.35, 1.0], // тот же цвет, что и в основном мире
                    });
                }
            }
        }
        
        // Показываем область видимости камеры (центр миникарты)
        let center_x = minimap_x + minimap_w / 2;
        let center_y = minimap_y + minimap_h / 2;
        
        // Показываем область видимости камеры в центре миникарты
        let transform = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new((cell_size * 2) as f32, (cell_size * 2) as f32, 1.0),
            glam::Quat::IDENTITY,
            glam::Vec3::new(center_x as f32, center_y as f32, 0.0),
        );
        
        self.minimap_instances.push(UIRect {
            model_matrix: transform.to_cols_array_2d(),
            color: [1.0, 1.0, 0.0, 0.5], // полупрозрачный желтый
        });
        
        // Буфер миникарты будет обновлен в render()
    }
    
    pub fn prepare_road_preview(
        &mut self,
        preview_path: &[glam::IVec2],
        is_building: bool,
        tile_atlas: &crate::atlas::TileAtlas,
    ) {
        use glam::{Mat4, Vec3};
        
        self.road_preview_instances.clear();
        
        if preview_path.is_empty() {
            return;
        }
        
        let half_w = tile_atlas.half_w as f32;
        let half_h = tile_atlas.half_h as f32;
        
        // Цвета для предпросмотра (как в CPU версии)
        let tint_color = if is_building {
            [0.47, 0.78, 0.47, 0.35] // зеленоватый для строительства
        } else {
            [0.78, 0.39, 0.39, 0.35] // красноватый для удаления
        };
        
        for &pos in preview_path.iter() {
            // ИЗОМЕТРИЧЕСКАЯ проекция В ПИКСЕЛЯХ (как у обычных дорог)
            let iso_x = (pos.x - pos.y) as f32 * half_w;
            let iso_y = ((pos.x + pos.y) as f32 * half_h) - half_h * 0.5;
            
            // Матрица трансформации дороги (размер как у тайла)
            let transform = Mat4::from_scale_rotation_translation(
                Vec3::new(half_w * 2.0, half_h * 2.0, 1.0),
                glam::Quat::IDENTITY,
                Vec3::new(iso_x, -iso_y, 0.0), // минус как у зданий
            );
            
            let instance = RoadInstance {
                model_matrix: transform.to_cols_array_2d(),
                road_mask: 0, // простая маска для предпросмотра
                tint_color,
                padding: [0; 3],
            };
            
            self.road_preview_instances.push(instance);
        }
        
        // Обновляем буфер инстансов предпросмотра дорог
        if !self.road_preview_instances.is_empty() {
            self.queue.write_buffer(
                &self.road_preview_instance_buffer,
                0,
                bytemuck::cast_slice(&self.road_preview_instances)
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
        
        // Обновляем UI буфер ДО начала render pass
        if !self.ui_rects.is_empty() {
            self.queue.write_buffer(
                &self.ui_rect_buffer,
                0,
                bytemuck::cast_slice(&self.ui_rects)
            );
        }
        
        // Обновляем буфер миникарты ДО начала render pass
        if !self.minimap_instances.is_empty() {
            self.queue.write_buffer(
                &self.minimap_buffer,
                0,
                bytemuck::cast_slice(&self.minimap_instances)
            );
        }
        
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
                
                // Рендерим дороги (сразу после тайлов, но перед зданиями)
                if !self.road_instances.is_empty() {
                    render_pass.set_pipeline(&self.road_render_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.building_vertex_buffer.slice(..)); // используем тот же quad
                    render_pass.set_vertex_buffer(1, self.road_instance_buffer.slice(..));
                    render_pass.set_index_buffer(self.building_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..self.road_instances.len() as u32);
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
                
                // Рендерим граждан (используем тот же pipeline что и для зданий)
                if !self.citizen_instances.is_empty() {
                    render_pass.set_pipeline(&self.building_render_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.building_vertex_buffer.slice(..)); // используем тот же quad
                    render_pass.set_vertex_buffer(1, self.citizen_instance_buffer.slice(..));
                    render_pass.set_index_buffer(self.building_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..self.citizen_instances.len() as u32);
                }
                
                // Рендерим предпросмотр дорог (поверх зданий и граждан)
                if !self.road_preview_instances.is_empty() {
                    render_pass.set_pipeline(&self.road_render_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.building_vertex_buffer.slice(..)); // используем тот же quad
                    render_pass.set_vertex_buffer(1, self.road_preview_instance_buffer.slice(..));
                    render_pass.set_index_buffer(self.building_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..self.road_preview_instances.len() as u32);
                }
                
                // Рендерим предпросмотр зданий (поверх всего остального)
                if !self.building_preview_instances.is_empty() {
                    render_pass.set_pipeline(&self.building_render_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.building_vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, self.building_preview_instance_buffer.slice(..));
                    render_pass.set_index_buffer(self.building_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..self.building_preview_instances.len() as u32);
                }
            }
            
            // Рендерим UI прямоугольники ПОСЛЕ карты (буфер уже обновлен до render pass)
            if !self.ui_rects.is_empty() {
                render_pass.set_pipeline(&self.ui_rect_render_pipeline);
                render_pass.set_bind_group(0, &self.screen_bind_group, &[]); // используем экранные координаты
                render_pass.set_vertex_buffer(0, self.tile_vertex_buffer.slice(..)); // используем тот же quad
                render_pass.set_vertex_buffer(1, self.ui_rect_buffer.slice(..));
                render_pass.set_index_buffer(self.tile_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..self.ui_rects.len() as u32);
            }
            
            // Рендерим миникарту (поверх UI)
            if !self.minimap_instances.is_empty() {
                render_pass.set_pipeline(&self.ui_rect_render_pipeline);
                render_pass.set_bind_group(0, &self.screen_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.tile_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.minimap_buffer.slice(..));
                render_pass.set_index_buffer(self.tile_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..self.minimap_instances.len() as u32);
            }
            
            // Рендерим погодные эффекты (поверх всего, кроме тултипов)
            if self.weather_uniform.weather_type != 0 {
                render_pass.set_pipeline(&self.weather_pipeline);
                render_pass.set_bind_group(0, &self.weather_bind_group, &[]);
                render_pass.draw(0..3, 0..1); // Полноэкранный треугольник
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
