use crate::atlas::TileAtlas;
use crate::render::wgpu_renderer::Vertex;
use crate::types::TileKind;
use wgpu::util::DeviceExt;

pub struct AtlasRenderer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub tile_size: f32,
    pub map_width: u32,
    pub map_height: u32,
    pub atlas: TileAtlas,
}

impl AtlasRenderer {
    pub fn new(device: &wgpu::Device, map_width: u32, map_height: u32, tile_size: f32) -> Self {
        let mut atlas = TileAtlas::new();
        atlas.ensure_zoom(2.0); // Начальный зум
        
        let vertices = Self::create_atlas_vertices(&atlas, map_width, map_height, tile_size);
        let indices = Self::create_atlas_indices(map_width, map_height);
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Index Buffer"),
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
            atlas,
        }
    }
    
    pub fn update_zoom(&mut self, device: &wgpu::Device, zoom: f32) {
        self.atlas.ensure_zoom(zoom);
        self.update_vertices(device);
    }
    
    pub fn update_world_data(&mut self, device: &wgpu::Device, world: &mut crate::world::World, cam_x: f32, cam_y: f32) {
        // Обновляем данные на основе World
        self.update_vertices_with_world(device, world, cam_x, cam_y);
    }
    
    fn update_vertices(&mut self, device: &wgpu::Device) {
        let vertices = Self::create_atlas_vertices(&self.atlas, self.map_width, self.map_height, self.tile_size);
        let indices = Self::create_atlas_indices(self.map_width, self.map_height);
        
        // Создаем новые буферы с обновленными данными
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.num_indices = indices.len() as u32;
    }
    
    fn update_vertices_with_world(&mut self, device: &wgpu::Device, world: &mut crate::world::World, cam_x: f32, cam_y: f32) {
        let mut vertices = Vec::new();
        
        // Определяем видимую область
        let visible_tiles_x = (self.map_width as f32 / self.tile_size) as i32;
        let visible_tiles_y = (self.map_height as f32 / self.tile_size) as i32;
        
        let start_x = (cam_x / self.tile_size) as i32 - visible_tiles_x / 2;
        let start_y = (cam_y / self.tile_size) as i32 - visible_tiles_y / 2;
        
        for y in 0..visible_tiles_y {
            for x in 0..visible_tiles_x {
                let world_x = start_x + x;
                let world_y = start_y + y;
                
                // Получаем тип тайла из World
                let tile_kind = world.get_tile(world_x, world_y);
                
                // Позиция тайла в мировых координатах
                let screen_x = (x as f32) * self.tile_size;
                let screen_y = (y as f32) * self.tile_size;
                
                // Цвет тайла в зависимости от типа
                let color = Self::get_tile_color(tile_kind);
                
                // Получаем UV координаты для типа тайла из атласа
                let (uv_min, uv_max) = Self::get_tile_uv_coordinates(tile_kind);
                
                // Создаем 4 вершины для тайла (квадрат)
                let tile_vertices = [
                    // Верхний левый
                    Vertex {
                        position: [screen_x, screen_y, 0.0],
                        uv: [uv_min.0, uv_max.1], // Поменяли Y координаты
                        color,
                    },
                    // Верхний правый
                    Vertex {
                        position: [screen_x + self.tile_size, screen_y, 0.0],
                        uv: [uv_max.0, uv_max.1], // Поменяли Y координаты
                        color,
                    },
                    // Нижний правый
                    Vertex {
                        position: [screen_x + self.tile_size, screen_y + self.tile_size, 0.0],
                        uv: [uv_max.0, uv_min.1], // Поменяли Y координаты
                        color,
                    },
                    // Нижний левый
                    Vertex {
                        position: [screen_x, screen_y + self.tile_size, 0.0],
                        uv: [uv_min.0, uv_min.1], // Поменяли Y координаты
                        color,
                    },
                ];
                
                vertices.extend_from_slice(&tile_vertices);
            }
        }
        
        let indices = Self::create_atlas_indices(visible_tiles_x as u32, visible_tiles_y as u32);
        
        // Создаем новые буферы с обновленными данными
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.num_indices = indices.len() as u32;
    }
    
    fn create_atlas_vertices(atlas: &TileAtlas, map_width: u32, map_height: u32, tile_size: f32) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        
        for y in 0..map_height {
            for x in 0..map_width {
                // Определяем тип тайла (пока простой паттерн)
                let tile_kind = Self::get_tile_kind(x, y);
                
                // Позиция тайла в мировых координатах
                let world_x = (x as f32) * tile_size;
                let world_y = (y as f32) * tile_size;
                
                // Цвет тайла в зависимости от типа
                let color = Self::get_tile_color(tile_kind);
                
                // Получаем UV координаты для типа тайла из атласа
                let (uv_min, uv_max) = Self::get_tile_uv_coordinates(tile_kind);
                
                // Создаем 4 вершины для тайла (квадрат)
                let tile_vertices = [
                    // Верхний левый
                    Vertex {
                        position: [world_x, world_y, 0.0],
                        uv: [uv_min.0, uv_max.1], // Поменяли Y координаты
                        color,
                    },
                    // Верхний правый
                    Vertex {
                        position: [world_x + tile_size, world_y, 0.0],
                        uv: [uv_max.0, uv_max.1], // Поменяли Y координаты
                        color,
                    },
                    // Нижний правый
                    Vertex {
                        position: [world_x + tile_size, world_y + tile_size, 0.0],
                        uv: [uv_max.0, uv_min.1], // Поменяли Y координаты
                        color,
                    },
                    // Нижний левый
                    Vertex {
                        position: [world_x, world_y + tile_size, 0.0],
                        uv: [uv_min.0, uv_min.1], // Поменяли Y координаты
                        color,
                    },
                ];
                
                vertices.extend_from_slice(&tile_vertices);
            }
        }
        
        vertices
    }
    
    fn create_atlas_indices(map_width: u32, map_height: u32) -> Vec<u16> {
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
    
    fn get_tile_kind(x: u32, y: u32) -> TileKind {
        // Простой паттерн для демонстрации
        match (x + y) % 3 {
            0 => TileKind::Grass,
            1 => TileKind::Forest,
            2 => TileKind::Water,
            _ => TileKind::Grass,
        }
    }
    
    fn get_tile_color(tile_kind: TileKind) -> [f32; 3] {
        match tile_kind {
            TileKind::Grass => [0.2, 0.8, 0.2],    // Зеленый
            TileKind::Forest => [0.1, 0.6, 0.1],   // Темно-зеленый
            TileKind::Water => [0.2, 0.4, 0.8],    // Синий
        }
    }
    
    // Метод для получения данных тайла из атласа
    pub fn get_tile_data(&self, tile_kind: TileKind) -> &[u8] {
        match tile_kind {
            TileKind::Grass => &self.atlas.grass,
            TileKind::Forest => &self.atlas.forest,
            TileKind::Water => {
                // Возвращаем первый кадр воды
                if !self.atlas.water_frames.is_empty() {
                    &self.atlas.water_frames[0]
                } else {
                    &self.atlas.grass // Fallback
                }
            }
        }
    }
    
    // Метод для получения UV координат тайла из атласа
    fn get_tile_uv_coordinates(tile_kind: TileKind) -> ((f32, f32), (f32, f32)) {
        // Spritesheet имеет размер 352x352 пикселей, содержит 11x11 тайлов (32x32 пикселя каждый)
        let tile_size = 32.0;
        let atlas_size = 352.0;
        let tiles_per_row = 11.0;
        
        let uv_tile_size = tile_size / atlas_size;
        
        match tile_kind {
            TileKind::Grass => {
                // Трава в строке 2 (индекс 2), первый тайл
                let x = 0.0;
                let y = 2.0;
                ((x * uv_tile_size, y * uv_tile_size), ((x + 1.0) * uv_tile_size, (y + 1.0) * uv_tile_size))
            }
            TileKind::Forest => {
                // Лес в строке 3 (индекс 3), первый тайл
                let x = 0.0;
                let y = 3.0;
                ((x * uv_tile_size, y * uv_tile_size), ((x + 1.0) * uv_tile_size, (y + 1.0) * uv_tile_size))
            }
            TileKind::Water => {
                // Вода в последней строке (индекс 7), первый тайл
                let x = 0.0;
                let y = 7.0;
                ((x * uv_tile_size, y * uv_tile_size), ((x + 1.0) * uv_tile_size, (y + 1.0) * uv_tile_size))
            }
        }
    }
}
