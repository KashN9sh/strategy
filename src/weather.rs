use rand::{rngs::StdRng, Rng};
use crate::types::WeatherKind;

/// Система управления погодой с автоматической сменой
pub struct WeatherSystem {
    current: WeatherKind,
    timer_ms: f32,
    next_change_ms: f32,
}

impl WeatherSystem {
    /// Создать новую систему погоды с заданным начальным состоянием
    pub fn new(initial: WeatherKind, rng: &mut StdRng) -> Self {
        let next_change = choose_weather_duration_ms(initial, rng);
        Self {
            current: initial,
            timer_ms: 0.0,
            next_change_ms: next_change,
        }
    }

    /// Обновить систему погоды (вызывать каждый кадр)
    pub fn update(&mut self, dt_ms: f32, rng: &mut StdRng) {
        self.timer_ms += dt_ms;
        if self.timer_ms >= self.next_change_ms {
            self.timer_ms = 0.0;
            self.current = pick_next_weather(self.current, rng);
            self.next_change_ms = choose_weather_duration_ms(self.current, rng);
        }
    }

    /// Получить текущую погоду
    pub fn current(&self) -> WeatherKind {
        self.current
    }

    /// Установить новую погоду (вручную, сбросив таймер)
    pub fn set(&mut self, new_weather: WeatherKind, rng: &mut StdRng) {
        self.current = new_weather;
        self.timer_ms = 0.0;
        self.next_change_ms = choose_weather_duration_ms(new_weather, rng);
    }

    /// Получить интенсивность погоды для эффектов рендеринга
    pub fn intensity(&self) -> f32 {
        match self.current {
            WeatherKind::Clear => 0.0,
            WeatherKind::Rain => 0.8,
            WeatherKind::Fog => 0.6,
            WeatherKind::Snow => 0.7,
        }
    }

    /// Получить метку и цвет погоды для UI
    pub fn ui_label_and_color(&self) -> (&'static [u8], [u8; 4]) {
        match self.current {
            WeatherKind::Clear => (b"CLEAR", [180, 200, 120, 255]),
            WeatherKind::Rain => (b"RAIN", [90, 120, 200, 255]),
            WeatherKind::Fog => (b"FOG", [160, 160, 160, 255]),
            WeatherKind::Snow => (b"SNOW", [220, 230, 255, 255]),
        }
    }
}

/// Выбрать длительность текущей погоды в миллисекундах
fn choose_weather_duration_ms(current: WeatherKind, rng: &mut StdRng) -> f32 {
    // Базовые интервалы (в секундах), затем добавляем разброс
    let (base_min, base_max) = match current {
        WeatherKind::Clear => (60.0, 120.0),
        WeatherKind::Rain => (40.0, 90.0),
        WeatherKind::Fog => (30.0, 70.0),
        WeatherKind::Snow => (50.0, 100.0),
    };
    let sec: f32 = rng.random_range(base_min..base_max);
    sec * 1000.0
}

/// Выбрать следующую погоду на основе вероятностных переходов
fn pick_next_weather(current: WeatherKind, rng: &mut StdRng) -> WeatherKind {
    // Вероятности переходов зависят от текущей погоды
    // Значения — веса; нормализуем автоматически
    let (opts, weights): (&[WeatherKind], &[f32]) = match current {
        WeatherKind::Clear => (
            &[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow],
            &[0.55, 0.25, 0.15, 0.05],
        ),
        WeatherKind::Rain => (
            &[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow],
            &[0.35, 0.35, 0.20, 0.10],
        ),
        WeatherKind::Fog => (
            &[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow],
            &[0.40, 0.20, 0.30, 0.10],
        ),
        WeatherKind::Snow => (
            &[WeatherKind::Clear, WeatherKind::Rain, WeatherKind::Fog, WeatherKind::Snow],
            &[0.30, 0.20, 0.10, 0.40],
        ),
    };
    let total: f32 = weights.iter().copied().sum();
    let mut r = rng.random_range(0.0..total);
    for (w, &p) in opts.iter().zip(weights.iter()) {
        if r < p {
            return *w;
        }
        r -= p;
    }
    *opts.last().unwrap_or(&current)
}

