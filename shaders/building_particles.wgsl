// Шейдер для частиц зданий (искры, дым, пыль)

// Униформы камеры
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct BuildingParticle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    life: f32,
    size: f32,
    color: vec4<f32>,
}

struct BuildingParticleUniform {
    time: f32,
    particle_count: u32,
    padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> particle_uniform: BuildingParticleUniform;

@group(0) @binding(2)
var<storage, read> particles: array<BuildingParticle>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) size: f32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let particle_index = vertex_index / 6u; // 6 вершин на частицу (2 треугольника)
    let vertex_in_triangle = vertex_index % 6u;
    
    if (particle_index >= particle_uniform.particle_count) {
        return VertexOutput(
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            0.0
        );
    }
    
    let particle = particles[particle_index];
    
    // Создаем квадрат для частицы
    var pos: vec2<f32>;
    if (vertex_in_triangle == 0u) {
        pos = vec2<f32>(-1.0, -1.0);
    } else if (vertex_in_triangle == 1u) {
        pos = vec2<f32>(1.0, -1.0);
    } else if (vertex_in_triangle == 2u) {
        pos = vec2<f32>(-1.0, 1.0);
    } else if (vertex_in_triangle == 3u) {
        pos = vec2<f32>(1.0, -1.0);
    } else if (vertex_in_triangle == 4u) {
        pos = vec2<f32>(1.0, 1.0);
    } else {
        pos = vec2<f32>(-1.0, 1.0);
    }
    
    // Масштабируем по размеру частицы
    pos *= particle.size;
    
    // Перемещаем в позицию частицы
    pos += particle.position;
    
    // Применяем матрицу камеры
    let world_pos = vec4<f32>(pos, 0.0, 1.0);
    let clip_pos = camera.view_proj * world_pos;
    
    return VertexOutput(
        clip_pos,
        particle.color,
        particle.size
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Простой круглый эффект для частиц
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(in.color.xy - center);
    let alpha = smoothstep(0.5, 0.0, dist) * in.color.a;
    
    return vec4<f32>(in.color.rgb, alpha);
}
