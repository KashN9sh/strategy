// Шейдер для UI спрайтов из props.png
// Использует экранные координаты (как ui_rect) и текстуру props (как resource)

struct ScreenUniform {
    screen_size: vec2<f32>, // ширина и высота экрана в пикселях
    padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> screen: ScreenUniform;

@group(1) @binding(0)
var props_texture: texture_2d<f32>;
@group(1) @binding(1)
var props_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct PropsInstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) props_id: u32,
    @location(7) tint_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) props_id: u32,
    @location(2) tint_color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: PropsInstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    var out: VertexOutput;
    
    // Трансформируем вершину моделью (получаем пиксельные экранные координаты)
    let world_pos = model_matrix * vec4<f32>(model.position, 1.0);
    
    // Конвертируем пиксельные координаты в NDC [-1, 1]
    // X: [0, width] -> [-1, 1]
    // Y: [0, height] -> [1, -1] (Y инвертирован в NDC)
    let ndc_x = (world_pos.x / screen.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (world_pos.y / screen.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = model.tex_coords;
    out.props_id = instance.props_id;
    out.tint_color = instance.tint_color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Специальный ID для полноразмерных текстур (не из атласа)
    let FULL_TEXTURE_ID: u32 = 0xFFFFFFFFu;
    
    var atlas_uv: vec2<f32>;
    
    // Если это полноразмерная текстура, используем UV координаты напрямую
    if (in.props_id == FULL_TEXTURE_ID) {
        // Для полноразмерных текстур используем UV координаты напрямую
        // В вершинах: tex_coords [0.0, 1.0] = левый верхний, [1.0, 0.0] = правый нижний
        // В текстурах: V=0 вверху, V=1 внизу
        // Поэтому инвертируем Y координату
        atlas_uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    } else {
        // props.png имеет сетку 5x4 спрайтов (5 колонок, 4 строки)
        // Каждый спрайт 16x16 пикселей, текстура 80x64 пикселей
        let sprites_per_row = 5.0;
        let sprites_per_col = 4.0;
        
        // Размер одного спрайта в UV координатах (0..1)
        let sprite_size_u = 1.0 / sprites_per_row; // 1.0 / 5.0 = 0.2
        let sprite_size_v = 1.0 / sprites_per_col; // 1.0 / 4.0 = 0.25
        
        // Вычисляем позицию спрайта в сетке (колонка и строка)
        let sprite_col = f32(in.props_id % 5u);
        let sprite_row = f32(in.props_id / 5u);
        
        // Вычисляем UV координаты для конкретного спрайта
        // in.uv идет от 0 до 1 для всего квада, масштабируем до размера одного спрайта в атласе
        // Базовые координаты спрайта в атласе (0..1)
        let base_u = sprite_col * sprite_size_u;
        let base_v = sprite_row * sprite_size_v;
        
        // Добавляем смещение внутри спрайта
        // В вершинах in.uv.y идет от 1 (верх) до 0 (низ), а в текстурах V=0 вверху
        // Поэтому используем (1.0 - in.uv.y) для правильной инверсии
        atlas_uv = vec2<f32>(
            base_u + in.uv.x * sprite_size_u,
            base_v + (1.0 - in.uv.y) * sprite_size_v
        );
    }
    
    // Сэмплируем текстуру
    let texture_color = textureSample(props_texture, props_sampler, atlas_uv);
    
    // Применяем тинт
    let final_color = texture_color * in.tint_color;
    
    return final_color;
}

