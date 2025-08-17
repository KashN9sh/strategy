@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) world_pos: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 1.0);
    out.uv = vertex.uv;
    out.color = vertex.color;
    out.world_pos = vertex.position.xy;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(t_diffuse, s_diffuse, in.uv);
    
    // Добавляем сетку
    let grid_size = 0.05; // Размер тайла
    let grid_width = 0.01; // Увеличиваем толщину линий сетки
    
    // Проверяем вертикальные линии (X координата)
    let vertical_line = fract(in.world_pos.x / grid_size) < grid_width;
    
    // Проверяем горизонтальные линии (Y координата) - делаем их более толстыми
    let horizontal_line = fract(in.world_pos.y / grid_size) < grid_width * 2.0;
    
    let is_grid_line = vertical_line || horizontal_line;
    
    // Разные цвета для вертикальных и горизонтальных линий
    let vertical_color = vec3<f32>(1.0, 0.0, 0.0); // Красный для вертикальных
    let horizontal_color = vec3<f32>(0.0, 1.0, 0.0); // Зеленый для горизонтальных
    
    let grid_color = select(vertical_color, horizontal_color, horizontal_line);
    let final_color = select(texture_color.rgb, grid_color, is_grid_line);
    let final_alpha = select(texture_color.a, 1.0, is_grid_line);
    
    return vec4<f32>(final_color, final_alpha);
}
