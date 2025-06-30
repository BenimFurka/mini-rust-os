use crate::vga_buffer::{Color};
use crate::print;

pub fn handle_screen_command(args: &[&str]) {
    if args.is_empty() || args[0] == "--help" {
        print_help();
        return;
    }

    match args[0] {
        "info" => show_screen_info(),
        "set" => handle_set_command(&args[1..]),
        "clear" => clear_screen(),
        _ => print!(("\nUnknown screen command. Type 'screen --help' for usage."), fg: Color::Red),
    }
}

fn show_screen_info() {
    // TODO: Это че за костыли, почему я такой леницев
    print!(("\nScreen Information:"), fg: Color::LightBlue);
    print!(("\n  Width: {} columns", 80), fg: Color::White);
    print!(("\n  Height: {} rows", 25), fg: Color::White);
    print!(("\n  Current position: (N/A, N/A)"), fg: Color::White);
    print!(("\n  Foreground color: N/A"), fg: Color::White);
    print!(("\n  Background color: N/A"), fg: Color::White);
}

fn handle_set_command(_args: &[&str]) {
    // TODO: Накорябал что-то не так в коде
    //       и теперь ОС ложится
    print!(("Doesn't work"), fg: Color::Yellow);

    /* 
    if args.len() < 2 {
        print!(("\nUsage: screen set <fg|bg> <color>"), fg: Color::Red);
        print_available_colors();
        return;
    }
    
    let param = args[0].to_lowercase();
    let value = args[1].to_lowercase();
    
    
    match param.as_str() {
        "fg" | "foreground" => {
            if let Some(color) = parse_color(&value) {
                use crate::vga_buffer::WRITER;
                WRITER.lock().set_foreground_color(color);
                print!(("\nForeground color set to {:?}", color), fg: color);
            } else {
                print!(("\nInvalid color. Available colors: "));
                print_available_colors();
            }
        }
        "bg" | "background" => {
            if let Some(color) = parse_color(&value) {
                use crate::vga_buffer::WRITER;
                let mut writer = WRITER.lock();
                writer.set_background_color(color);
                writer.clear_screen();
                print!(("\nBackground color set to {:?}", color));
            } else {
                print!(("\nInvalid color. Available colors: "));
                print_available_colors();
            }
        }
        _ => print!(("\nUnknown parameter '{}'", param), fg: Color::Red),
    }
    */
}

fn clear_screen() {
    crate::vga_buffer::clear_screen();
    print!(("\n"));
}

fn _parse_color(color: &str) -> Option<Color> {
    match color.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "blue" => Some(Color::Blue),
        "green" => Some(Color::Green),
        "cyan" => Some(Color::Cyan),
        "red" => Some(Color::Red),
        "magenta" => Some(Color::Magenta),
        "brown" => Some(Color::Brown),
        "light_gray" | "lightgray" | "light_grey" | "lightgrey" => Some(Color::LightGray),
        "dark_gray" | "darkgray" | "dark_grey" | "darkgrey" => Some(Color::DarkGray),
        "light_blue" | "lightblue" => Some(Color::LightBlue),
        "light_green" | "lightgreen" => Some(Color::LightGreen),
        "light_cyan" | "lightcyan" => Some(Color::LightCyan),
        "light_red" | "lightred" => Some(Color::LightRed),
        "pink" | "light_magenta" | "lightmagenta" => Some(Color::Pink),
        "yellow" => Some(Color::Yellow),
        "white" => Some(Color::White),
        "k" | "blk" => Some(Color::Black),
        "b" | "blu" => Some(Color::Blue),
        "g" | "grn" => Some(Color::Green),
        "c" | "cyn" => Some(Color::Cyan),
        "r" => Some(Color::Red),
        "m" | "mag" => Some(Color::Magenta),
        "br" | "brn" => Some(Color::Brown),
        "lg" | "lgr" => Some(Color::LightGray),
        "dg" | "dgr" => Some(Color::DarkGray),
        "lb" | "lbl" => Some(Color::LightBlue),
        "lgn" => Some(Color::LightGreen),
        "lc" | "lcy" => Some(Color::LightCyan),
        "lr" | "lrd" => Some(Color::LightRed),
        "y" | "ylw" => Some(Color::Yellow),
        "w" | "wht" => Some(Color::White),
        _ => None,
    }
}

fn print_available_colors() {
    let colors = [
        ("black (k, blk)", Color::Black),
        ("blue (b, blu)", Color::Blue),
        ("green (g, grn)", Color::Green),
        ("cyan (c, cyn)", Color::Cyan),
        ("red (r, red)", Color::Red),
        ("magenta (m, mag)", Color::Magenta),
        ("brown (br, brn)", Color::Brown),
        ("light_gray (lg, lgr)", Color::LightGray),
        ("dark_gray (dg, dgr)", Color::DarkGray),
        ("light_blue (lb, lbl)", Color::LightBlue),
        ("light_green (lg, lgn)", Color::LightGreen),
        ("light_cyan (lc, lcy)", Color::LightCyan),
        ("light_red (lr, lrd)", Color::LightRed),
        ("pink/light_magenta (pink, m, mag)", Color::Pink),
        ("yellow (y, ylw)", Color::Yellow),
        ("white (w, wht)", Color::White),
    ];
    
    print!(("\nAvailable colors (aliases in parentheses):"), fg: Color::LightBlue);
    for (i, (name, color)) in colors.iter().enumerate() {
        print!(("  {:<40}", name), fg: *color);
        if i % 2 == 1 {
            print!(("\n"));
        }
    }
}

fn print_help() {
    print!(("\nScreen commands:"), fg: Color::LightBlue);
    print!(("\n  screen info                - Show screen status"), fg: Color::White);
    print!(("\n  screen set fg <color>     - Set foreground color"), fg: Color::White);
    print!(("\n  screen set bg <color>     - Set background color"), fg: Color::White);
    print!(("\n  screen clear              - Clear the screen"), fg: Color::White);
    print!(("\n  screen --help             - Show this help message"), fg: Color::White);
    print!(("\n"));
    print_available_colors();
}
