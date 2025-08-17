use crate::types::TileKind;
use crate::render::wgpu_renderer::Vertex;
use wgpu::util::DeviceExt;

pub struct TileRenderer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub tile_size: f32,
    pub map_width: u32,
    pub map_height: u32,
}

impl TileRenderer {
    pub fn new(
        device: &wgpu::Device,
        map_width: u32,
        map_height: u32,
        tile_size: f32,
    ) -> Self {
        let vertices = Self::create_tile_vertices(map_width, map_height, tile_size);
        let indices = Self::create_tile_indices(map_width, map_height);
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            tile_size,
            map_width,
            map_height,
        }
    }
    
    fn create_tile_vertices(map_width: u32, map_height: u32, tile_size: f32) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        
        for y in 0..map_height {
            for x in 0..map_width {
                // Позиция тайла в мировых координатах
                let world_x = (x as f32 - map_width as f32 / 2.0) * tile_size;
                let world_y = (y as f32 - map_height as f32 / 2.0) * tile_size;
                
                // Создаем 4 вершины для каждого тайла (квадрат)
                let tile_vertices = [
                    // Нижний левый
                    Vertex {
                        position: [world_x, world_y, 0.0],
                        uv: [0.0, 0.0],
                        color: [1.0, 1.0, 1.0],
                    },
                    // Нижний правый
                    Vertex {
                        position: [world_x + tile_size, world_y, 0.0],
                        uv: [1.0, 0.0],
                        color: [1.0, 1.0, 1.0],
                    },
                    // Верхний правый
                    Vertex {
                        position: [world_x + tile_size, world_y + tile_size, 0.0],
                        uv: [1.0, 1.0],
                        color: [1.0, 1.0, 1.0],
                    },
                    // Верхний левый
                    Vertex {
                        position: [world_x, world_y + tile_size, 0.0],
                        uv: [0.0, 1.0],
                        color: [1.0, 1.0, 1.0],
                    },
                ];
                
                vertices.extend_from_slice(&tile_vertices);
            }
        }
        
        vertices
    }
    
    fn create_tile_indices(map_width: u32, map_height: u32) -> Vec<u16> {
        let mut indices = Vec::new();
        
        for tile_index in 0..(map_width * map_height) {
            let base_vertex = (tile_index * 4) as u16;
            
            // Первый треугольник
            indices.push(base_vertex);
            indices.push(base_vertex + 1);
            indices.push(base_vertex + 2);
            
            // Второй треугольник
            indices.push(base_vertex);
            indices.push(base_vertex + 2);
            indices.push(base_vertex + 3);
        }
        
        indices
    }
    
    pub fn get_tile_color(tile_kind: TileKind) -> [f32; 3] {
        match tile_kind {
            TileKind::Grass => [0.2, 0.8, 0.2],     // Зеленый
            TileKind::Forest => [0.1, 0.5, 0.1],    // Темно-зеленый
            TileKind::Water => [0.2, 0.4, 0.8],     // Синий
        }
    }
    
    pub fn update_tile_colors(&self, _device: &wgpu::Device, _queue: &wgpu::Queue, _tiles: &[TileKind]) {
        // Пока просто заглушка - позже добавим обновление цветов тайлов
        // Это потребует создания новых буферов или использования uniform буферов
    }
}
