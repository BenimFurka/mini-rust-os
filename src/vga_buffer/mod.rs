#![allow(dead_code)]

use core::fmt;
use volatile::Volatile;
use core::ptr::NonNull;
use lazy_static::lazy_static;
use crate::spin::SpinMutex;

mod modes;
pub use modes::{VideoMode, VideoState};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl Color {
    pub fn from_u8(value: u8) -> Color {
        match value {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::Pink,
            14 => Color::Yellow,
            15 => Color::White,
            _ => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn foreground(&self) -> Color {
        Color::from_u8(self.0 & 0x0F)
    }
    
    pub fn background(&self) -> Color {
        Color::from_u8((self.0 >> 4) & 0x0F)
    }

    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const DEFAULT_HEIGHT: usize = 25;
const DEFAULT_WIDTH: usize = 80;

pub(crate) struct Buffer {
    chars: [[Volatile<ScreenChar>; DEFAULT_WIDTH]; DEFAULT_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    video_state: VideoState,
}

unsafe impl Send for Writer {}
unsafe impl Sync for Writer {}

impl Writer {
    pub fn new() -> Self {
        let mut video_state = VideoState::default();
        video_state.set_mode(VideoMode::Text80x25);
        Self {
            column_position: 0,
            color_code: ColorCode::new(Color::LightGreen, Color::Black),
            video_state,
        }
    }
    
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color_code = ColorCode::new(fg, bg);
    }
    
    pub fn set_video_mode(&mut self, mode: VideoMode) {
        self.video_state.set_mode(mode);
        self.column_position = 0;
    }
    
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= self.video_state.width {
                    self.new_line();
                }
                let row = self.video_state.height - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                
                unsafe {
                    self.video_state.buffer.as_mut().chars[row][col].write(ScreenChar {
                        ascii_character: byte,
                        color_code,
                    });
                }
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..self.video_state.height {
            for col in 0..self.video_state.width {
                let character = unsafe { self.video_state.buffer.as_ref().chars[row][col].read() };
                unsafe {
                    self.video_state.buffer.as_mut().chars[row - 1][col].write(character);
                }
            }
        }
        self.clear_row(self.video_state.height - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..self.video_state.width {
            unsafe {
                self.video_state.buffer.as_mut().chars[row][col].write(blank);
            }
        }
    }

    pub fn delete_char(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            let row = self.video_state.height - 1;
            let col = self.column_position;
            
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            
            unsafe {
                self.video_state.buffer.as_mut().chars[row][col].write(blank);
            }
        } else if self.video_state.height > 1 {
            self.clear_row(self.video_state.height - 1);
            self.column_position = self.video_state.width - 1;
            
            self.delete_char();
        }
    }
    
    pub fn set_foreground_color(&mut self, color: Color) {
        self.color_code = ColorCode::new(color, self.color_code.background());
    }
    
    pub fn set_background_color(&mut self, color: Color) {
        self.color_code = ColorCode::new(self.color_code.foreground(), color);
    }
    
    pub fn clear_screen(&mut self) {
        for row in 0..self.video_state.height {
            self.clear_row(row);
        }
        self.column_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: SpinMutex<Writer> = SpinMutex::new(Writer::new());
}

pub static WRITER_LOCK: SpinMutex<()> = SpinMutex::new(());

pub fn print(args: fmt::Arguments, fg: Option<Color>, bg: Option<Color>) {
    use core::fmt::Write;
    let _lock = WRITER_LOCK.lock();
    let mut writer = WRITER.lock();

    if let Some(fg_color) = fg {
        let current_bg = (writer.color_code.0 >> 4) as u8;
        writer.set_color(fg_color, Color::from_u8(current_bg));
    }
    if let Some(bg_color) = bg {
        let current_fg = (writer.color_code.0 & 0x0F) as u8;
        writer.set_color(Color::from_u8(current_fg), bg_color);
    }
    
    let _ = writer.write_fmt(args);
    writer.set_color(Color::LightGreen, Color::Black);
}

pub fn clear_screen() {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    for _ in 0..writer.video_state.height {
        let _ = writer.write_str("\n");
    }
}

pub fn set_video_mode(mode: VideoMode) {
    let mut writer = WRITER.lock();
    writer.set_video_mode(mode);
    clear_screen();
}
