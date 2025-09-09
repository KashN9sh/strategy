// Шейдер для UI элементов и спрайтов

struct CameraUniform {
    view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,
    padding: vec2<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var s_texture: sampler;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Преобразуем координаты экрана в NDC
    let ndc_x = (model.position.x / camera.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (model.position.y / camera.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_texture, s_texture, in.tex_coords);
    return tex_color * in.color;
}
