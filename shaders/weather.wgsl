// Погодные эффекты - шейдер для дождя, снега, тумана

struct WeatherUniform {
    weather_type: u32, // 0=Clear, 1=Rain, 2=Fog, 3=Snow
    time: f32,
    intensity: f32,
    padding: f32,
}

struct ScreenUniform {
    screen_size: vec2<f32>,
    padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> weather: WeatherUniform;

@group(0) @binding(1)
var<uniform> screen: ScreenUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) screen_pos: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Полноэкранный треугольник
    var pos: vec2<f32>;
    if (vertex_index == 0u) {
        pos = vec2<f32>(-1.0, -1.0);
    } else if (vertex_index == 1u) {
        pos = vec2<f32>(3.0, -1.0);
    } else {
        pos = vec2<f32>(-1.0, 3.0);
    }
    
    return VertexOutput(
        vec4<f32>(pos, 0.0, 1.0),
        pos * 0.5 + 0.5
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.screen_pos;
    let time = weather.time;
    
    // Базовый цвет (прозрачный)
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    if (weather.weather_type == 0u) {
        // Clear - никаких эффектов
        return color;
    }
    
    if (weather.weather_type == 1u) {
        // Rain - синие капли
        let rain_intensity = weather.intensity;
        
        // Создаем паттерн дождя
        let rain_scale = 0.1;
        let rain_speed = 2.0;
        let rain_offset = time * rain_speed;
        
        // Вертикальные полосы дождя
        let rain_x = uv.x * 20.0 + rain_offset;
        let rain_y = uv.y * 30.0 + rain_offset * 0.7;
        
        let rain_noise = fract(sin(rain_x) * 43758.5453) * fract(sin(rain_y) * 43758.5453);
        let rain_strength = smoothstep(0.8, 1.0, rain_noise) * rain_intensity;
        
        if (rain_strength > 0.1) {
            color = vec4<f32>(0.3, 0.5, 0.8, rain_strength * 0.6);
        }
    }
    
    if (weather.weather_type == 2u) {
        // Fog - серый туман
        let fog_intensity = weather.intensity;
        
        // Создаем волнообразный туман
        let fog_scale = 0.05;
        let fog_speed = 0.5;
        let fog_offset = time * fog_speed;
        
        let fog_x = uv.x * 10.0 + fog_offset;
        let fog_y = uv.y * 10.0 + fog_offset * 0.3;
        
        let fog_noise = sin(fog_x) * cos(fog_y) * 0.5 + 0.5;
        let fog_strength = fog_noise * fog_intensity;
        
        if (fog_strength > 0.1) {
            color = vec4<f32>(0.7, 0.7, 0.7, fog_strength * 0.4);
        }
    }
    
    if (weather.weather_type == 3u) {
        // Snow - белые снежинки
        let snow_intensity = weather.intensity;
        
        // Создаем паттерн снега
        let snow_scale = 0.08;
        let snow_speed = 0.8;
        let snow_offset = time * snow_speed;
        
        let snow_x = uv.x * 15.0 + snow_offset;
        let snow_y = uv.y * 20.0 + snow_offset * 0.5;
        
        let snow_noise = fract(sin(snow_x) * 43758.5453) * fract(sin(snow_y) * 43758.5453);
        let snow_strength = smoothstep(0.7, 1.0, snow_noise) * snow_intensity;
        
        if (snow_strength > 0.1) {
            color = vec4<f32>(0.9, 0.9, 1.0, snow_strength * 0.8);
        }
    }
    
    return color;
}
