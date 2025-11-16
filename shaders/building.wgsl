// Базовый шейдер для рендеринга зданий

// Униформы камеры
struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Текстура spritesheet для тайлов и зданий
@group(1) @binding(0)
var t_spritesheet: texture_2d<f32>;
@group(1) @binding(1)
var s_spritesheet: sampler;

// Текстура деревьев
@group(1) @binding(2)
var t_trees: texture_2d<f32>;
@group(1) @binding(3)
var s_trees: sampler;

// Текстура зданий
@group(1) @binding(4)
var t_buildings: texture_2d<f32>;
@group(1) @binding(5)
var s_buildings: sampler;


// Входящие данные вершины
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// Входящие данные инстанса зданий
struct BuildingInstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) building_id: u32,
    @location(7) tint_color: vec4<f32>,
}

// Входящие данные инстанса дорог
struct RoadInstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) road_mask: u32,
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

// Вершинный шейдер для зданий
@vertex
fn vs_main_building(
    model: VertexInput,
    instance: BuildingInstanceInput,
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

// Вершинный шейдер для дорог
@vertex
fn vs_main_road(
    model: VertexInput,
    instance: RoadInstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.building_id = 200 + instance.road_mask; // специальный ID для дорог (200+)
    out.tint_color = instance.tint_color;
    out.model_pos = model.position.xy;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}


// Фрагментный шейдер
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let building_id = in.building_id;
    
    
    // Деревья (ID 100-102 = стадии 0-2) - рендерим из текстуры
    if building_id >= 100u && building_id <= 102u {
        let stage = building_id - 100u;
        
        // Текстура trees.png содержит 3 спрайта по горизонтали (0, 1, 2)
        let tree_sprites = 3.0; // количество спрайтов
        let sprite_x = f32(stage); // номер спрайта по горизонтали
        
        // Вычисляем UV координаты для конкретного спрайта дерева
        let u = (sprite_x + in.tex_coords.x) / tree_sprites;
        let v = in.tex_coords.y;
        
        // Сэмплируем текстуру деревьев
        let tree_color = textureSample(t_trees, s_trees, vec2<f32>(u, v));
        
        // Отбрасываем полностью прозрачные пиксели
        if tree_color.a < 0.01 {
            discard;
        }
        
        // Применяем tint_color
        return tree_color * in.tint_color;
    }
    
    // Обычные здания (ID 0-12) - рендерим из текстуры
    if building_id <= 12u {
        // Текстура buildings.png содержит спрайты зданий
        let building_sprites = 8.0; // количество спрайтов в текстуре
        let sprite_x = f32(building_id % 8u); // используем доступные спрайты
        
        // Вычисляем UV координаты для конкретного спрайта здания
        let u = (sprite_x + in.tex_coords.x) / building_sprites;
        let v = in.tex_coords.y;
        
        // Сэмплируем текстуру зданий
        let building_color = textureSample(t_buildings, s_buildings, vec2<f32>(u, v));
        
        // Отбрасываем полностью прозрачные пиксели
        if building_color.a < 0.01 {
            discard;
        }
        
        // Применяем tint_color
        return building_color * in.tint_color;
    }
    
    // Дороги (ID 200-215) - рендерим процедурно
    if building_id >= 200u && building_id <= 215u {
        let road_mask = building_id - 200u;
        
        // Процедурная отрисовка дороги на основе маски соединений
        // Пока используем простую заливку, позже можно добавить текстуру
        let road_color = vec3<f32>(0.47, 0.43, 0.35); // коричневатый цвет дороги
        
        // Изометрическая форма дороги (ромб)
        let iso_x = in.model_pos.x;
        let iso_y = in.model_pos.y;
        // Изометрическая проверка: |x| + |y| <= 0.5
        if abs(iso_x) + abs(iso_y) > 0.5 {
            discard;
        }
        
        // Применяем tint_color
        return vec4<f32>(road_color, 1.0) * in.tint_color;
    }
    
    // Неизвестный ID - серый по умолчанию
    return vec4<f32>(0.5, 0.5, 0.5, 1.0) * in.tint_color;
}
