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
        // Rain - синие капли под углом 45 градусов
        let rain_intensity = weather.intensity;
        
        // Скорость дождя
        let rain_speed = 5.0;
        let rain_offset = fract(time * rain_speed);
        
        // Поворачиваем координаты на 45 градусов
        let angle = 0.3; // 45 градусов в радианах
        let cos_a = cos(angle);
        let sin_a = sin(angle);
        
        // Поворачиваем UV координаты
        let rotated_x = uv.x * cos_a - uv.y * sin_a;
        let rotated_y = uv.x * sin_a + uv.y * cos_a;
        
        // Создаем наклонные линии дождя
        let rain_x = fract(rotated_x * 100.0);
        let rain_y = fract(rotated_y * 40.0 + rain_offset);
        
        // Наклонные линии дождя (больше частиц)
        let is_rain = step(0.5, rain_x) * rain_y;
        let rain_strength = is_rain * rain_intensity;
        
        if (rain_strength > 0.05) {
            color = vec4<f32>(0.2, 0.4, 1.0, rain_strength * 0.9);
        }
    }
    
    if (weather.weather_type == 2u) {
        // Fog - серый туман
        let fog_intensity = weather.intensity;
        
        // Простой волнообразный туман
        let fog_speed = 0.3;
        let fog_offset = time * fog_speed;
        
        let fog_x = uv.x * 8.0 + fog_offset;
        let fog_y = uv.y * 8.0 + fog_offset * 0.5;
        
        let fog_noise = sin(fog_x) * cos(fog_y) * 0.5 + 0.5;
        let fog_strength = fog_noise * fog_intensity;
        
        if (fog_strength > 0.05) {
            color = vec4<f32>(0.6, 0.6, 0.6, fog_strength * 0.6);
        }
    }
    
    if (weather.weather_type == 3u) {
        // Snow - белые снежинки под углом 45 градусов
        let snow_intensity = weather.intensity;
        
        // Скорость снега
        let snow_speed = 1.0;
        let snow_offset = fract(time * snow_speed);
        
        // Поворачиваем координаты на 45 градусов
        let angle = 0.3; // 45 градусов в радианах
        let cos_a = cos(angle);
        let sin_a = sin(angle);
        
        // Поворачиваем UV координаты
        let rotated_x = uv.x * cos_a - uv.y * sin_a;
        let rotated_y = uv.x * sin_a + uv.y * cos_a;
        
        // Создаем наклонные снежинки
        let snow_grid_x = fract(rotated_x * 40.0 + sin(rotated_y * 10.0 + time) * 0.2);
        let snow_grid_y = fract(rotated_y * 50.0 + snow_offset);
        
        // Наклонные снежинки как точки (больше частиц)
        let is_snowflake = step(0.7, snow_grid_x) * step(0.7, snow_grid_y);
        let snow_strength = is_snowflake * snow_intensity;
        
        if (snow_strength > 0.1) {
            color = vec4<f32>(0.95, 0.95, 1.0, snow_strength);
        }
    }
    
    return color;
}
