// Шейдер для мягкого радиального свечения (ночное освещение)

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_position: vec2<f32>,
    zoom: f32,
    padding: f32,
}

struct LightInstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) radius: f32,
    @location(7) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) model_pos: vec2<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    instance: LightInstanceInput,
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
    out.clip_position = camera.view_proj * world_pos;
    
    // Передаем позицию в локальных координатах (для вычисления расстояния от центра)
    out.model_pos = vec2<f32>(position.x, position.y);
    
    // Передаем радиус и цвет
    out.radius = instance.radius;
    out.color = instance.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Вычисляем расстояние от центра (0, 0) до текущего пикселя
    let dist = length(in.model_pos);
    
    // Радиальный градиент с мягким затуханием
    // Используем smoothstep для плавного затухания от центра к краю
    let fade_start = in.radius * 0.6;
    let fade_end = in.radius;
    let alpha = 1.0 - smoothstep(fade_start, fade_end, dist);
    
    // Если пиксель вне радиуса, отбрасываем его
    if dist > in.radius {
        discard;
    }
    
    // Применяем альфа к цвету
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

