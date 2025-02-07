use core::fmt;
use volatile::Volatile;

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
struct ColorCode(u8);

impl ColorCode {
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

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

use core::ptr::NonNull;

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: NonNull<Buffer>,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                unsafe {
                    self.buffer.as_mut().chars[row][col].write(ScreenChar {
                        ascii_character: byte,
                        color_code,
                    });
                }
                self.column_position += 1;
            }
        }
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = unsafe { self.buffer.as_ref().chars[row][col].read() };
                unsafe {
                    self.buffer.as_mut().chars[row - 1][col].write(character);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            unsafe {
                self.buffer.as_mut().chars[row][col].write(blank);
            }
        }
    }
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }
    pub fn delete_char(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1; 
            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
    
            unsafe {
                self.buffer.as_mut().chars[row][col].write(ScreenChar {
                    ascii_character: b' ', 
                    color_code: self.color_code,
                });
            }
        } else {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = unsafe { self.buffer.as_ref().chars[row][col].read() };
                    unsafe {
                        self.buffer.as_mut().chars[row - 1][col].write(character);
                    }
                }
            }
    
            self.clear_row(BUFFER_HEIGHT - 1);
    
            self.column_position = BUFFER_WIDTH - 1;
        }
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

use crate::spin::Spinlock;

pub static WRITER_LOCK: Spinlock<> = Spinlock::new();

pub static mut WRITER: Writer = Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::LightGreen, Color::Black),
    buffer: unsafe { NonNull::new_unchecked(0xb8000 as *mut _) },
};


macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), None, None);
    });
    ($($arg:tt)*, fg: $fg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), Some($fg), None);
    });
    ($($arg:tt)*, bg: $bg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), None, Some($bg));
    });
    ($($arg:tt)*, fg: $fg:expr, bg: $bg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), Some($fg), Some($bg));
    });
}
pub fn print(args: fmt::Arguments, fg: Option<Color>, bg: Option<Color>) {
    use core::fmt::Write;
    WRITER_LOCK.lock(); 

        
    unsafe {
        if let Some(fg_color) = fg {
            if let Some(bg_color) = bg {
                WRITER.set_color(fg_color, bg_color);
            } else {
                let current_bg = (WRITER.color_code.0 >> 4) as u8;
                WRITER.set_color(fg_color, Color::from_u8(current_bg));
            }
        } else if let Some(bg_color) = bg {
            let current_fg = (WRITER.color_code.0 & 0x0F) as u8;
            WRITER.set_color(Color::from_u8(current_fg), bg_color);
        }
        WRITER.write_fmt(args).unwrap();
    }
    WRITER_LOCK.unlock();

}

pub fn clear_screen() {
    for _ in 0..BUFFER_HEIGHT {
        print!("");
    }
}

