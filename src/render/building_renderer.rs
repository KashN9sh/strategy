use crate::types::{Building, BuildingKind};
use crate::render::wgpu_renderer::Vertex;
use wgpu::util::DeviceExt;

pub struct BuildingRenderer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub tile_size: f32,
    pub buildings: Vec<Building>,
}

impl BuildingRenderer {
    pub fn new(device: &wgpu::Device, tile_size: f32) -> Self {
        let vertices = Self::create_building_vertices(&[], tile_size);
        let indices = Self::create_building_indices(0);
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            tile_size,
            buildings: Vec::new(),
        }
    }
    
    pub fn update_buildings(&mut self, device: &wgpu::Device, buildings: Vec<Building>) {
        self.buildings = buildings;
        let vertices = Self::create_building_vertices(&self.buildings, self.tile_size);
        let indices = Self::create_building_indices(self.buildings.len());
        
        // Создаем новые буферы с обновленными данными
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.num_indices = indices.len() as u32;
    }
    
    fn create_building_vertices(buildings: &[Building], tile_size: f32) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        
        for building in buildings {
            // Позиция здания в мировых координатах
            let world_x = (building.pos.x as f32) * tile_size;
            let world_y = (building.pos.y as f32) * tile_size;
            
            // Размер здания (пока все здания 1x1)
            let building_size = tile_size * 0.8; // Немного меньше тайла
            let offset = (tile_size - building_size) / 2.0;
            
            // Цвет здания в зависимости от типа
            let color = Self::get_building_color(building.kind);
            
            // Создаем 4 вершины для здания (квадрат)
            let building_vertices = [
                // Нижний левый
                Vertex {
                    position: [world_x + offset, world_y + offset, 0.1], // Z = 0.1 чтобы быть выше тайлов
                    uv: [0.0, 0.0],
                    color,
                },
                // Нижний правый
                Vertex {
                    position: [world_x + offset + building_size, world_y + offset, 0.1],
                    uv: [1.0, 0.0],
                    color,
                },
                // Верхний правый
                Vertex {
                    position: [world_x + offset + building_size, world_y + offset + building_size, 0.1],
                    uv: [1.0, 1.0],
                    color,
                },
                // Верхний левый
                Vertex {
                    position: [world_x + offset, world_y + offset + building_size, 0.1],
                    uv: [0.0, 1.0],
                    color,
                },
            ];
            
            vertices.extend_from_slice(&building_vertices);
        }
        
        vertices
    }
    
    fn create_building_indices(num_buildings: usize) -> Vec<u16> {
        let mut indices = Vec::new();
        
        for building_index in 0..num_buildings {
            let base_vertex = (building_index * 4) as u16;
            
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
    
    pub fn get_building_color(building_kind: BuildingKind) -> [f32; 3] {
        match building_kind {
            BuildingKind::Lumberjack => [0.8, 0.4, 0.2],    // Коричневый
            BuildingKind::House => [0.9, 0.9, 0.7],         // Светло-желтый
            BuildingKind::Warehouse => [0.6, 0.6, 0.8],     // Серо-синий
            BuildingKind::Forester => [0.3, 0.7, 0.3],      // Зеленый
            BuildingKind::StoneQuarry => [0.7, 0.7, 0.7],   // Серый
            BuildingKind::ClayPit => [0.8, 0.6, 0.4],       // Коричневый
            BuildingKind::Kiln => [0.8, 0.3, 0.3],          // Красный
            BuildingKind::WheatField => [0.9, 0.8, 0.3],    // Желтый
            BuildingKind::Mill => [0.6, 0.5, 0.3],          // Коричневый
            BuildingKind::Bakery => [0.9, 0.7, 0.5],        // Светло-коричневый
            BuildingKind::Fishery => [0.3, 0.5, 0.8],       // Синий
            BuildingKind::IronMine => [0.5, 0.5, 0.5],      // Темно-серый
            BuildingKind::Smelter => [0.7, 0.4, 0.4],       // Темно-красный
        }
    }
}
