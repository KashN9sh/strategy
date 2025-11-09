use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use anyhow::Result;
use rand::Rng;

/// Менеджер фоновой музыки
pub struct MusicManager {
    sink: Sink,
    _stream: OutputStream,
    tracks: Vec<PathBuf>,
    current_track_index: usize,
    volume: f32,
    enabled: bool,
}

impl MusicManager {
    /// Создать новый менеджер музыки
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        // Найти все треки в папке assets/tracks
        let tracks_dir = Path::new("assets/tracks");
        let mut tracks = Vec::new();
        
        if tracks_dir.exists() {
            let entries = std::fs::read_dir(tracks_dir)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wav") {
                    tracks.push(path);
                }
            }
        }
        
        // Сортировать треки для предсказуемого порядка
        tracks.sort();
        
        let mut manager = Self {
            sink,
            _stream,
            tracks,
            current_track_index: 0,
            volume: 0.5,
            enabled: true,
        };
        
        // Начать воспроизведение первого трека, если есть
        if !manager.tracks.is_empty() {
            manager.play_current_track()?;
        }
        
        Ok(manager)
    }
    
    /// Воспроизвести текущий трек
    fn play_current_track(&mut self) -> Result<()> {
        if self.tracks.is_empty() || !self.enabled {
            return Ok(());
        }
        
        let track_path = &self.tracks[self.current_track_index];
        let file = File::open(track_path)?;
        let source = Decoder::new(BufReader::new(file))?;
        
        // Установить громкость
        let source = source.amplify(self.volume);
        
        // Очистить предыдущий трек и добавить новый
        self.sink.stop();
        self.sink.append(source);
        
        Ok(())
    }
    
    /// Обновить состояние музыки (проверить, закончился ли трек)
    pub fn update(&mut self) -> Result<()> {
        if !self.enabled || self.tracks.is_empty() {
            return Ok(());
        }
        
        // Если трек закончился, переключить на следующий
        if self.sink.empty() {
            self.next_track()?;
        }
        
        Ok(())
    }
    
    /// Переключить на следующий трек
    pub fn next_track(&mut self) -> Result<()> {
        if self.tracks.is_empty() {
            return Ok(());
        }
        
        self.current_track_index = (self.current_track_index + 1) % self.tracks.len();
        self.play_current_track()?;
        
        Ok(())
    }
    
    /// Переключить на случайный трек
    pub fn random_track(&mut self, rng: &mut impl Rng) -> Result<()> {
        if self.tracks.is_empty() {
            return Ok(());
        }
        
        if self.tracks.len() > 1 {
            let new_index = rng.random_range(0..self.tracks.len());
            // Убедимся, что не выбрали тот же трек
            if new_index == self.current_track_index {
                self.current_track_index = (new_index + 1) % self.tracks.len();
            } else {
                self.current_track_index = new_index;
            }
        }
        
        self.play_current_track()?;
        
        Ok(())
    }
    
    /// Установить громкость (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        // Обновить громкость текущего трека
        if !self.sink.empty() {
            // Остановить и перезапустить с новой громкостью
            let was_playing = !self.sink.empty();
            if was_playing {
                self.sink.stop();
                if let Ok(_) = self.play_current_track() {
                    // Трек перезапущен
                }
            }
        }
    }
    
    /// Получить текущую громкость
    pub fn volume(&self) -> f32 {
        self.volume
    }
    
    /// Включить/выключить музыку
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.sink.stop();
        } else if self.tracks.is_empty() {
            // Попробуем перезагрузить треки
            if let Ok(_) = self.reload_tracks() {
                let _ = self.play_current_track();
            }
        } else {
            let _ = self.play_current_track();
        }
    }
    
    /// Проверить, включена ли музыка
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Перезагрузить список треков
    fn reload_tracks(&mut self) -> Result<()> {
        let tracks_dir = Path::new("assets/tracks");
        let mut tracks = Vec::new();
        
        if tracks_dir.exists() {
            let entries = std::fs::read_dir(tracks_dir)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wav") {
                    tracks.push(path);
                }
            }
        }
        
        tracks.sort();
        self.tracks = tracks;
        
        Ok(())
    }
    
    /// Получить имя текущего трека
    pub fn current_track_name(&self) -> Option<String> {
        self.tracks.get(self.current_track_index)
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

