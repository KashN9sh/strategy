// Простой шейдер для отображения props.png с масштабированием

struct ViewUniform {
    zoom: f32,
    offset_x: f32,
    offset_y: f32,
    padding: f32,
}

@group(0) @binding(0)
var<uniform> view: ViewUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@group(0) @binding(1)
var texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Применяем масштаб и смещение к UV координатам
    // Центрируем UV (0.5, 0.5) и масштабируем относительно центра
    let centered_uv = in.uv - vec2<f32>(0.5, 0.5);
    let scaled_uv = centered_uv / view.zoom + vec2<f32>(0.5, 0.5);
    let final_uv = scaled_uv + vec2<f32>(view.offset_x, view.offset_y);
    
    return textureSample(texture, texture_sampler, final_uv);
}

