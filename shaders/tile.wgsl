// Базовый шейдер для рендеринга тайлов

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_position: vec2<f32>,
    zoom: f32,
    padding: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) tile_id: u32,
    @location(7) tint_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tile_id: f32,
    @location(2) tint_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_spritesheet: texture_2d<f32>;

@group(1) @binding(1)
var s_spritesheet: sampler;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    out.tile_id = f32(instance.tile_id);
    out.tint_color = instance.tint_color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Вычисляем UV координаты для спрайтшита
    // Реальный размер: 352x352 пикселей = сетка 11x11 спрайтов по 32x32
    let sprites_per_row = 11.0;
    let sprite_rows = 11.0;
    
    let sprite_width = 1.0 / sprites_per_row;
    let sprite_height = 1.0 / sprite_rows;
    
    // Определяем позицию спрайта в атласе для каждого типа тайла
    var sprite_x: f32;
    var sprite_y: f32;
    
    if (in.tile_id == 0.0) {
        // Трава - row 2 согласно CPU коду
        sprite_x = 0.0; // первый столбец в ряду
        sprite_y = 2.0; // row 2
    } else if (in.tile_id == 1.0) {
        // Лес - пробуем найти в том же ряду что трава
        sprite_x = 3.0; // попробуем 4-й столбец
        sprite_y = 2.0; // row 2
    } else if (in.tile_id == 2.0) {
        // Вода - последний ряд в сетке 11x11
        sprite_x = 0.0;
        sprite_y = 10.0; // последний ряд (индекс 10)
    } else {
        // По умолчанию - первый тайл
        sprite_x = 0.0;
        sprite_y = 0.0;
    }
    
    // Преобразуем в UV координаты
    let uv_x = (sprite_x + in.tex_coords.x) * sprite_width;
    let uv_y = (sprite_y + in.tex_coords.y) * sprite_height;
    
    // Сэмплируем текстуру
    let tex_color = textureSample(t_spritesheet, s_spritesheet, vec2<f32>(uv_x, uv_y));
    
    // Применяем тинт
    return tex_color * in.tint_color;
}
