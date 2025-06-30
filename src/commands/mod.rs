mod cpu;
mod disk;
mod mem;
mod pic;
mod port;
mod screen;
mod system;

use crate::vga_buffer::Color;
use crate::print;

pub fn handle_command(command: &str, args: &[&str]) {
    match command {
        "help" => print_help(),
        "pic" => pic::handle_pic_command(args),
        "port" => port::handle_port_command(args),
        "cpu" => cpu::handle_cpu_command(args),
        "mem" => mem::handle_mem_command(args),
        "disk" => disk::handle_disk_command(args),
        "screen" => screen::handle_screen_command(args),
        "reboot" => system::reboot(),
        "shutdown" => system::shutdown(),
        _ => print!(("\nUnknown command: {}. Type 'help' for available commands.", command), fg: Color::Red),
    }
}

fn print_help() {
    print!(("\nAvailable commands:"), fg: Color::LightBlue);
    print!(("\n  cpu     - CPU information and control"), fg: Color::White);
    print!(("\n  disk    - Disk operations and information"), fg: Color::White);
    print!(("\n  mem     - Memory operations"), fg: Color::White);
    print!(("\n  pic     - Programmable Interrupt Controller control"), fg: Color::White);
    print!(("\n  port    - Port I/O operations"), fg: Color::White);
    print!(("\n  screen  - Screen control"), fg: Color::White);
    print!(("\n  reboot  - Reboot the system"), fg: Color::White);
    print!(("\n  shutdown - Shut down the system"), fg: Color::White);
    print!(("\n  help    - Show this help"), fg: Color::White);
}
