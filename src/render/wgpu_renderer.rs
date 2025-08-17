use anyhow::Result;
use wgpu::util::DeviceExt;
use winit::window::Window;
use std::sync::Arc;
use crate::render::texture_manager::TextureManager;
use crate::render::tile_renderer::TileRenderer;
use crate::render::building_renderer::BuildingRenderer;
use crate::render::ui_renderer::UIRenderer;
use crate::render::atlas_renderer::AtlasRenderer;

pub struct WgpuRenderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    // Шейдеры
    pub render_pipeline: wgpu::RenderPipeline,
    pub colored_pipeline: wgpu::RenderPipeline, // Pipeline for colored objects (buildings)
    pub grid_pipeline: wgpu::RenderPipeline, // Pipeline for grid rendering
    pub bind_group_layout: wgpu::BindGroupLayout,
    
    // Буферы
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    
    // Window и instance для создания surface
    pub window: Arc<Window>,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    
    // Texture manager
    pub texture_manager: TextureManager,
    
    // Tile renderer
    pub tile_renderer: Option<TileRenderer>,
    
    // Building renderer
    pub building_renderer: Option<BuildingRenderer>,
    
    // UI renderer
    pub ui_renderer: Option<UIRenderer>,
    
    // Atlas renderer
    pub atlas_renderer: Option<AtlasRenderer>,
}

impl WgpuRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        
        // Создаем инстанс
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        
        // Создаем временную поверхность для получения адаптера
        let window_clone = window.clone();
        let temp_surface = instance.create_surface(window_clone.as_ref())?;
        
        // Получаем адаптер
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&temp_surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find an appropriate adapter"))?;
        
        // Создаем устройство и очередь
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        
        // Конфигурируем поверхность
        let surface_caps = temp_surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        // Создаем шейдеры
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/textured.wgsl").into()),
        });
        
        // Создаем буферы
        let vertices = create_quad_vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let indices = create_quad_indices();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Создаем bind group layout для текстур
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Текстура
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Сэмплер
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        // Создаем pipeline
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
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
        
        // Создаем colored pipeline для зданий
        let colored_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Colored Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/colored.wgsl").into()),
        });
        
        let colored_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Colored Pipeline"),
            layout: None, // Не нужен bind group layout для цветных объектов
            vertex: wgpu::VertexState {
                module: &colored_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &colored_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
        
        // Создаем grid pipeline для отображения сетки
        let grid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/grid.wgsl").into()),
        });
        
        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &grid_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &grid_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
        
        // Создаем texture manager
        let texture_manager = TextureManager::new();
        
        Ok(Self {
            device,
            queue,
            surface_config,
            size,
            render_pipeline,
            colored_pipeline,
            grid_pipeline,
            bind_group_layout,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            window,
            instance,
            adapter,
            texture_manager,
            tile_renderer: None,
            building_renderer: None,
            ui_renderer: None,
            atlas_renderer: None,
        })
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
        }
    }
    
    pub fn create_tile_renderer(&mut self, map_width: u32, map_height: u32, tile_size: f32) {
        self.tile_renderer = Some(TileRenderer::new(&self.device, map_width, map_height, tile_size));
    }

    pub fn create_building_renderer(&mut self, tile_size: f32) {
        self.building_renderer = Some(BuildingRenderer::new(&self.device, tile_size));
    }

    pub fn create_ui_renderer(&mut self, screen_width: f32, screen_height: f32) {
        self.ui_renderer = Some(UIRenderer::new(&self.device, screen_width, screen_height));
    }

    pub fn create_atlas_renderer(&mut self, map_width: u32, map_height: u32, tile_size: f32) {
        self.atlas_renderer = Some(AtlasRenderer::new(&self.device, map_width, map_height, tile_size));
    }
    
    pub fn update_atlas_world_data(&mut self, world: &mut crate::world::World, cam_x: f32, cam_y: f32) {
        if let Some(ref mut atlas_renderer) = self.atlas_renderer {
            atlas_renderer.update_world_data(&self.device, world, cam_x, cam_y);
        }
    }

    pub fn create_bind_group(&self, texture_name: &str) -> Option<wgpu::BindGroup> {
        if let Some(texture) = self.texture_manager.get_texture(texture_name) {
            Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some(&format!("{}_bind_group", texture_name)),
            }))
        } else {
            None
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Создаем поверхность для каждого кадра
        let surface = self.instance.create_surface(self.window.as_ref()).map_err(|_| wgpu::SurfaceError::Lost)?;
        surface.configure(&self.device, &self.surface_config);
        
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Создаем bind group для spritesheet
        let bind_group = self.create_bind_group("spritesheet");
        if bind_group.is_some() {
            println!("✅ Bind group создан успешно");
        } else {
            println!("❌ Bind group не создан!");
        }
        
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
            
            render_pass.set_pipeline(&self.render_pipeline);
            
            // Рендерим тайлы используя atlas renderer с сеткой
            if let Some(ref atlas_renderer) = self.atlas_renderer {
                // Используем grid pipeline для тайлов с сеткой
                render_pass.set_pipeline(&self.grid_pipeline);
                
                // Устанавливаем bind group для spritesheet
                if let Some(ref bind_group) = bind_group {
                    render_pass.set_bind_group(0, bind_group, &[]);
                }
                
                render_pass.set_vertex_buffer(0, atlas_renderer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(atlas_renderer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..atlas_renderer.num_indices, 0, 0..1);
            }
                
                // Рендерим здания поверх тайлов
                if let Some(ref building_renderer) = self.building_renderer {
                    // Переключаемся на colored pipeline для зданий
                    render_pass.set_pipeline(&self.colored_pipeline);
                    
                    render_pass.set_vertex_buffer(0, building_renderer.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(building_renderer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..building_renderer.num_indices, 0, 0..1);
                    
                    // Возвращаемся к основному pipeline для остальных объектов
                    render_pass.set_pipeline(&self.render_pipeline);
                }
                
                // Рендерим UI поверх всего остального
                if let Some(ref ui_renderer) = self.ui_renderer {
                    // Переключаемся на colored pipeline для UI
                    render_pass.set_pipeline(&self.colored_pipeline);
                    
                    render_pass.set_vertex_buffer(0, ui_renderer.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(ui_renderer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..ui_renderer.num_indices, 0, 0..1);
                    
                    // Возвращаемся к основному pipeline
                    render_pass.set_pipeline(&self.render_pipeline);
                }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
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
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn create_quad_vertices() -> [Vertex; 4] {
    [
        Vertex {
            position: [-0.5, -0.5, 0.0],
            uv: [0.0, 0.0],
            color: [1.0, 1.0, 1.0], // Белый цвет для лучшей видимости текстуры
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            uv: [1.0, 0.0],
            color: [1.0, 1.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.0],
            uv: [1.0, 1.0],
            color: [1.0, 1.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.0],
            uv: [0.0, 1.0],
            color: [1.0, 1.0, 1.0],
        },
    ]
}

fn create_quad_indices() -> [u16; 6] {
    [0, 1, 2, 0, 2, 3]
}
