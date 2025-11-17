use serde::{Serialize, Deserialize};

/// Тип уведомления
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationKind {
    ResearchCompleted { name: String },
    BuildingUnlocked { name: String },
    Warning { message: String },
    Info { message: String },
}

/// Уведомление
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub kind: NotificationKind,
    pub timer_ms: f32,
    pub max_time_ms: f32,
}

impl Notification {
    /// Создать новое уведомление
    pub fn new(kind: NotificationKind) -> Self {
        Self {
            kind,
            timer_ms: 5000.0,  // 5 секунд по умолчанию
            max_time_ms: 5000.0,
        }
    }
    
    /// Обновить таймер уведомления
    pub fn update(&mut self, delta_ms: f32) -> bool {
        self.timer_ms -= delta_ms;
        self.timer_ms > 0.0
    }
    
    /// Получить текст уведомления
    pub fn text(&self) -> String {
        match &self.kind {
            NotificationKind::ResearchCompleted { name } => {
                format!("Research completed: {}", name)
            }
            NotificationKind::BuildingUnlocked { name } => {
                format!("Unlocked: {}", name)
            }
            NotificationKind::Warning { message } => {
                format!("Warning: {}", message)
            }
            NotificationKind::Info { message } => {
                message.clone()
            }
        }
    }
    
    /// Получить прозрачность уведомления (для fade out эффекта)
    pub fn alpha(&self) -> f32 {
        let fade_duration = 1000.0; // Последняя секунда - fade out
        if self.timer_ms < fade_duration {
            self.timer_ms / fade_duration
        } else {
            1.0
        }
    }
}

/// Система уведомлений
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NotificationSystem {
    pub notifications: Vec<Notification>,
}

impl NotificationSystem {
    /// Создать новую систему уведомлений
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }
    
    /// Добавить уведомление
    pub fn add(&mut self, kind: NotificationKind) {
        self.notifications.push(Notification::new(kind));
    }
    
    /// Обновить все уведомления
    pub fn update(&mut self, delta_ms: f32) {
        self.notifications.retain_mut(|n| n.update(delta_ms));
    }
    
    /// Очистить все уведомления
    pub fn clear(&mut self) {
        self.notifications.clear();
    }
}

