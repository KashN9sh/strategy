// Базовый шейдер для UI прямоугольников (панели, кнопки, иконки)
// UI рендерится в экранных пиксельных координатах, не зависит от мировой камеры

struct ScreenUniform {
    screen_size: vec2<f32>, // ширина и высота экрана в пикселях
    padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> screen: ScreenUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

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
    
    // Трансформируем вершину моделью (получаем пиксельные экранные координаты)
    let world_pos = model_matrix * vec4<f32>(model.position, 1.0);
    
    // Конвертируем пиксельные координаты в NDC [-1, 1]
    // X: [0, width] -> [-1, 1]
    // Y: [0, height] -> [1, -1] (Y инвертирован в NDC)
    let ndc_x = (world_pos.x / screen.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (world_pos.y / screen.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = instance.color;
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Простой цветной прямоугольник
    return in.color;
}
