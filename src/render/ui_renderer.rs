use crate::render::wgpu_renderer::Vertex;
use wgpu::util::DeviceExt;

#[derive(Debug, Clone)]
pub struct UIElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 3],
    pub element_type: UIElementType,
}

#[derive(Debug, Clone)]
pub enum UIElementType {
    Button { text: String, is_pressed: bool },
    Panel,
    Text { content: String },
    Icon { texture_name: String },
}

pub struct UIRenderer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub elements: Vec<UIElement>,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl UIRenderer {
    pub fn new(device: &wgpu::Device, screen_width: f32, screen_height: f32) -> Self {
        let vertices = Self::create_ui_vertices(&[], screen_width, screen_height);
        let indices = Self::create_ui_indices(0);
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            elements: Vec::new(),
            screen_width,
            screen_height,
        }
    }
    
    pub fn update_elements(&mut self, device: &wgpu::Device, elements: Vec<UIElement>) {
        self.elements = elements;
        let vertices = Self::create_ui_vertices(&self.elements, self.screen_width, self.screen_height);
        let indices = Self::create_ui_indices(self.elements.len());
        
        // Создаем новые буферы с обновленными данными
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.num_indices = indices.len() as u32;
    }
    
    pub fn resize(&mut self, device: &wgpu::Device, new_width: f32, new_height: f32) {
        self.screen_width = new_width;
        self.screen_height = new_height;
        
        // Обновляем буферы с новыми размерами экрана
        let vertices = Self::create_ui_vertices(&self.elements, self.screen_width, self.screen_height);
        let indices = Self::create_ui_indices(self.elements.len());
        
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.num_indices = indices.len() as u32;
    }
    
    fn create_ui_vertices(elements: &[UIElement], screen_width: f32, screen_height: f32) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        
        for element in elements {
            // Конвертируем координаты экрана в нормализованные координаты (-1 до 1)
            let x1 = (element.x / screen_width) * 2.0 - 1.0;
            let y1 = 1.0 - (element.y / screen_height) * 2.0; // Инвертируем Y
            let x2 = ((element.x + element.width) / screen_width) * 2.0 - 1.0;
            let y2 = 1.0 - ((element.y + element.height) / screen_height) * 2.0;
            
            // Z = 0.2 чтобы быть выше зданий
            let z = 0.2;
            
            // Создаем 4 вершины для UI элемента (квадрат)
            let element_vertices = [
                // Нижний левый
                Vertex {
                    position: [x1, y2, z],
                    uv: [0.0, 0.0],
                    color: element.color,
                },
                // Нижний правый
                Vertex {
                    position: [x2, y2, z],
                    uv: [1.0, 0.0],
                    color: element.color,
                },
                // Верхний правый
                Vertex {
                    position: [x2, y1, z],
                    uv: [1.0, 1.0],
                    color: element.color,
                },
                // Верхний левый
                Vertex {
                    position: [x1, y1, z],
                    uv: [0.0, 1.0],
                    color: element.color,
                },
            ];
            
            vertices.extend_from_slice(&element_vertices);
        }
        
        vertices
    }
    
    fn create_ui_indices(num_elements: usize) -> Vec<u16> {
        let mut indices = Vec::new();
        
        for element_index in 0..num_elements {
            let base_vertex = (element_index * 4) as u16;
            
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
    
    // Вспомогательные методы для создания UI элементов
    pub fn create_button(x: f32, y: f32, width: f32, height: f32, text: String) -> UIElement {
        UIElement {
            x,
            y,
            width,
            height,
            color: [0.3, 0.3, 0.8], // Синий цвет для кнопок
            element_type: UIElementType::Button { text, is_pressed: false },
        }
    }
    
    pub fn create_panel(x: f32, y: f32, width: f32, height: f32) -> UIElement {
        UIElement {
            x,
            y,
            width,
            height,
            color: [0.2, 0.2, 0.2], // Темно-серый для панелей
            element_type: UIElementType::Panel,
        }
    }
    
    pub fn create_text_element(x: f32, y: f32, width: f32, height: f32, content: String) -> UIElement {
        UIElement {
            x,
            y,
            width,
            height,
            color: [1.0, 1.0, 1.0], // Белый для текста
            element_type: UIElementType::Text { content },
        }
    }
}
