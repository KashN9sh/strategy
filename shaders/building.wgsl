// Базовый шейдер для рендеринга зданий

// Униформы камеры
struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Текстура spritesheet
@group(1) @binding(0)
var t_spritesheet: texture_2d<f32>;
@group(1) @binding(1)
var s_spritesheet: sampler;

// Входящие данные вершины
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// Входящие данные инстанса
struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) building_id: u32,
    @location(7) tint_color: vec4<f32>,
}

// Выходящие данные вершинного шейдера
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) building_id: u32,
    @location(2) tint_color: vec4<f32>,
    @location(3) model_pos: vec2<f32>, // позиция в модельном пространстве для кружка
}

// Вершинный шейдер
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
    out.tex_coords = model.tex_coords;
    out.building_id = instance.building_id;
    out.tint_color = instance.tint_color;
    out.model_pos = model.position.xy; // сохраняем позицию для отрисовки кружка
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Фрагментный шейдер
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let building_id = in.building_id;
    
    // building_id = 255 означает гражданина (рендерим как кружок)
    if building_id == 255u {
        // Отрисовка кружка: отбрасываем пиксели вне радиуса 0.5
        let dist = length(in.model_pos);
        if dist > 0.5 {
            discard;
        }
        // Сглаживание краев кружка
        let alpha = smoothstep(0.5, 0.45, dist);
        return vec4<f32>(in.tint_color.rgb, in.tint_color.a * alpha);
    }
    
    // Обычные здания
    var color: vec4<f32>;
    
    // Цвета зданий по типу (соответствует BuildingKind enum)
    switch building_id {
        case 0u: { // House
            color = vec4<f32>(0.8, 0.6, 0.4, 1.0); // коричневый
        }
        case 1u: { // Lumberjack
            color = vec4<f32>(0.6, 0.4, 0.2, 1.0); // темно-коричневый
        }
        case 2u: { // Warehouse
            color = vec4<f32>(0.6, 0.6, 0.6, 1.0); // серый
        }
        case 3u: { // Forester
            color = vec4<f32>(0.4, 0.7, 0.3, 1.0); // зеленый
        }
        case 4u: { // StoneQuarry
            color = vec4<f32>(0.7, 0.7, 0.6, 1.0); // светло-серый
        }
        case 5u: { // ClayPit
            color = vec4<f32>(0.8, 0.5, 0.3, 1.0); // глиняный
        }
        case 6u: { // Kiln
            color = vec4<f32>(0.9, 0.4, 0.2, 1.0); // красный
        }
        case 7u: { // WheatField
            color = vec4<f32>(0.9, 0.8, 0.3, 1.0); // желтый
        }
        case 8u: { // Mill
            color = vec4<f32>(0.9, 0.9, 0.8, 1.0); // белый
        }
        case 9u: { // Bakery
            color = vec4<f32>(0.8, 0.7, 0.5, 1.0); // песочный
        }
        case 10u: { // Fishery
            color = vec4<f32>(0.4, 0.6, 0.8, 1.0); // голубой
        }
        case 11u: { // IronMine
            color = vec4<f32>(0.3, 0.3, 0.3, 1.0); // темно-серый
        }
        case 12u: { // Smelter
            color = vec4<f32>(0.8, 0.3, 0.1, 1.0); // оранжевый
        }
        default: {
            color = vec4<f32>(0.5, 0.5, 0.5, 1.0); // серый по умолчанию
        }
    }
    
    // Применяем tint_color
    color = color * in.tint_color;
    
    return color;
}
