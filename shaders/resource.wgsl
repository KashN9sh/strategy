// Шейдер для рендеринга ресурсов (поленья, камни, железо)
// Использует тот же vertex buffer что и здания, но с отдельной логикой

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_position: vec2<f32>,
    zoom: f32,
    padding: f32,
}

struct ResourceInstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) resource_id: u32,
    @location(7) tint_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) resource_id: u32,
    @location(3) tint_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var texture_atlas: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    instance: ResourceInstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Собираем матрицу модели из отдельных векторов
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );
    
    // Применяем матрицу модели к позиции вершины
    let world_pos = model_matrix * vec4<f32>(position, 1.0);
    out.world_position = world_pos.xyz;
    out.clip_position = camera.view_proj * world_pos;
    
    // Передаем UV координаты
    out.uv = tex_coords;
    
    // Передаем данные инстанса
    out.resource_id = instance.resource_id;
    out.tint_color = instance.tint_color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Специальный ID для простых цветных прямоугольников (без текстуры)
    // Используем максимальное значение u32 как флаг "только цвет"
    let SOLID_COLOR_ID = 0xFFFFFFFFu;
    
    // Если resource_id = SOLID_COLOR_ID, просто возвращаем цвет без текстуры
    if (in.resource_id == SOLID_COLOR_ID) {
        return in.tint_color;
    }
    
    // Иначе используем текстуру из атласа
    // Размер атласа (11x11 спрайтов)
    let sprites_per_row = 11.0;
    let sprite_size = 1.0 / sprites_per_row;
    
    // Вычисляем UV координаты для конкретного спрайта
    let sprite_x = f32(in.resource_id % 11u);
    let sprite_y = f32(in.resource_id / 11u);
    
    let atlas_uv = vec2<f32>(
        (sprite_x + in.uv.x) * sprite_size,
        (sprite_y + in.uv.y) * sprite_size
    );
    
    // Сэмплируем текстуру
    let texture_color = textureSample(texture_atlas, texture_sampler, atlas_uv);
    
    // Применяем тинт
    let final_color = texture_color * in.tint_color;
    
    return final_color;
}
