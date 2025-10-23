// Шейдер для рендеринга граждан с эмоциями

// Униформы камеры
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

// Униформы экрана
struct ScreenUniform {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

// Входящие данные вершины
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// Входящие данные инстанса граждан (с эмоциями)
struct CitizenInstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) building_id: u32,
    @location(7) tint_color: vec4<f32>,
    @location(8) emotion: u32,
    @location(9) state: u32,
}

// Выходящие данные вершинного шейдера
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) building_id: u32,
    @location(2) tint_color: vec4<f32>,
    @location(3) model_pos: vec2<f32>, // позиция в модельном пространстве для кружка
    @location(4) emotion: u32,
    @location(5) state: u32,
}

// Униформы
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> screen: ScreenUniform;

// Текстура лиц для эмоций граждан
@group(1) @binding(0)
var t_faces: texture_2d<f32>;
@group(1) @binding(1)
var s_faces: sampler;

// Вершинный шейдер для граждан
@vertex
fn vs_main_citizen(
    model: VertexInput,
    instance: CitizenInstanceInput,
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
    out.emotion = instance.emotion;
    out.state = instance.state;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Фрагментный шейдер
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let building_id = in.building_id;
    
    // building_id = 255 означает гражданина (рендерим как кружок с лицом)
    if building_id == 255u {
        // Отрисовка кружка: отбрасываем пиксели вне радиуса 0.5
        let dist = length(in.model_pos);
        if dist > 0.5 {
            discard;
        }
        
        // Сглаживание краев кружка
        let alpha = smoothstep(0.5, 0.45, dist);
        
        // Рендерим лицо в центре кружка
        let face_size = 0.3; // размер лица относительно кружка
        let face_center = vec2<f32>(0.0, 0.0);
        let face_dist = length(in.model_pos - face_center);
        
        if face_dist <= face_size {
            // Вычисляем UV координаты для лица
            let face_uv = (in.model_pos - face_center) / face_size * 0.5 + 0.5;
            
            // Определяем какой спрайт лица использовать (0=sad, 1=neutral, 2=happy)
            let face_sprite = f32(in.emotion);
            let face_sprites = 3.0; // количество спрайтов в текстуре
            
            // UV координаты для конкретного спрайта лица
            let u = (face_sprite + face_uv.x) / face_sprites;
            let v = 1.0 - face_uv.y; // инвертируем V координату чтобы лица не были вверх ногами
            
            // Сэмплируем текстуру лиц
            let face_color = textureSample(t_faces, s_faces, vec2<f32>(u, v));
            
            // Если есть пиксели лица, используем их
            if face_color.a > 0.1 {
                return vec4<f32>(face_color.rgb, face_color.a);
            }
        }
        
        // Иначе рисуем обычный кружок
        return vec4<f32>(in.tint_color.rgb, in.tint_color.a);
    }
    
    // Для других объектов (не должно происходить в этом шейдере)
    return vec4<f32>(1.0, 0.0, 1.0, 1.0); // magenta для отладки
}
