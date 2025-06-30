//! Я не знаю как оставить TODO на весь файл, но
//! TODO: Сделать что-то нормальное, или хотя-бы лучше чем это
//!       добавить чего-то нового короче

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoMode {
    Text80x25,
}

impl VideoMode {
    pub fn dimensions(&self) -> (usize, usize) {
        match self {
            VideoMode::Text80x25 => (80, 25),
        }
    }

    pub fn memory_address(&self) -> usize {
        match self {
            VideoMode::Text80x25 => 0xb8000,
        }
    }
}

pub struct VideoState {
    pub mode: VideoMode,
    pub width: usize,
    pub height: usize,
    pub buffer: NonNull<Buffer>,
}

unsafe impl Send for VideoState {}
unsafe impl Sync for VideoState {}

impl Default for VideoState {
    fn default() -> Self {
        Self::new(VideoMode::Text80x25)
    }
}

impl VideoState {
    pub fn new(mode: VideoMode) -> Self {
        let (width, height) = mode.dimensions();
        let buffer_ptr = mode.memory_address() as *mut Buffer;
        
        Self {
            mode,
            width,
            height,
            buffer: unsafe { NonNull::new_unchecked(buffer_ptr) },
        }
    }
    
    pub fn set_mode(&mut self, mode: VideoMode) {
        *self = Self::new(mode);
    }
}
