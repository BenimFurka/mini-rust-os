use crate::vga_buffer::Color;

macro_rules! print {
    (($($arg:tt)*)) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), None, None);
    });
    (($($arg:tt)*), fg: $fg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), Some($fg), None);
    });
    (($($arg:tt)*), bg: $bg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), None, Some($bg));
    });
    (($($arg:tt)*), fg: $fg:expr, bg: $bg:expr) => ({
        $crate::vga_buffer::print(format_args!($($arg)*), Some($fg), Some($bg));
    });
}


pub fn handle_calc(args: &[u8]) {
    let args_str = core::str::from_utf8(args).unwrap_or("").trim();

    if args_str.is_empty() {
        print!(("\nUsage: calc [a] [op] [b]"), fg: Color::Yellow);
        return;
    }

    match evaluate_expression(args_str) {
        Ok(result) => print!(("\nResult: {}", result), fg: Color::LightCyan),
        Err(err) => print!(("\nError: {}", err), fg: Color::Red),
    }
}


fn evaluate_expression(expr: &str) -> Result<i32, &str> {
    let mut parts = expr.split_whitespace();
    let a: i32 = parts.next().unwrap_or("").parse().map_err(|_| "Invalid number")?;
    let op = parts.next().ok_or("Missing operator")?;
    let b: i32 = parts.next().unwrap_or("").parse().map_err(|_| "Invalid number")?;

    let result = match op {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => {
            if b == 0 {
                return Err("Division by zero");
            }
            a / b
        }
        _ => return Err("Unknown operator"),
    };

    Ok(result)
}