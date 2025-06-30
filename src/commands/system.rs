use x86_64::instructions::port::Port;
use core::arch::asm;
use crate::vga_buffer::Color;
use crate::print;

pub fn reboot() {
    print!(("\nRebooting..."), fg: Color::Yellow);
    
    unsafe {
        let mut port = Port::new(0x64);
        
        while port.read() & 0x02 != 0 {}
        
        port.write(0xFE as u8);
    }
    
    triple_fault_reboot();
}

pub fn shutdown() {
    print!(("\nShutting down..."), fg: Color::Yellow);
    
    unsafe {
        let mut port = Port::new(0x604);
        port.write(0x2000 as u16);
        
        let mut port = Port::new(0xB004);
        port.write(0x2000 as u16);
        
        let mut port = Port::new(0x4004);
        port.write(0x3400 as u16);
    }
    
    print!(("\nShutdown failed. The system may not support ACPI shutdown."), fg: Color::Red);
    print!(("\nYou may need to manually power off the system."), fg: Color::White);
}

fn triple_fault_reboot() {
    unsafe {
        asm!(
            "cli",
            "lidt [{}]",
            in(reg) 0 as u64,
            options(nomem, nostack)
        );
        
        asm!(
            "xor rax, rax",
            "div rax",
            out("rax") _,
            options(nomem, nostack)
        );
    }
    
    print!(("\nReboot failed. The system may be in an unstable state."), fg: Color::Red);
}
