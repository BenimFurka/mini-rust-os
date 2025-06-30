#![allow(dead_code)]

use crate::vga_buffer::Color;
use crate::print;
use core::arch::asm;

const MAX_STRING_LEN: usize = 48;

struct CpuInfo {
    vendor: [u8; 16],
    vendor_len: usize,
    brand: [u8; MAX_STRING_LEN],
    brand_len: usize,
    features: [&'static str; 16],
    features_len: usize,
    cores: u32,
    threads: u32,
    family: u8,
    model: u8,
    stepping: u8,
    max_cpuid: u32,
    max_ext_cpuid: u32,
}

fn get_cpuid_info() -> CpuInfo {
    let mut info = CpuInfo {
        vendor: [0; 16],
        vendor_len: 0,
        brand: [0; MAX_STRING_LEN],
        brand_len: 0,
        features: [""; 16],
        features_len: 0,
        cores: 1,
        threads: 1,
        family: 0,
        model: 0,
        stepping: 0,
        max_cpuid: 0,
        max_ext_cpuid: 0,
    };
    
    unsafe {
        let (max_basic, ebx, ecx, edx) = cpuid(0);
        info.max_cpuid = max_basic;
        
        if max_basic >= 1 {
            let (eax, _, _, _) = cpuid(1);
            
            info.stepping = (eax & 0xF) as u8;
            info.model = ((eax >> 4) & 0xF) as u8;
            let ext_model = ((eax >> 16) & 0xF) as u8;
            let ext_family = ((eax >> 8) & 0xF) as u8;
            
            info.family = ext_family + ((eax >> 20) & 0xFF) as u8;
            info.model |= ext_model << 4;
        }
        
        let (max_extended, _, _, _) = cpuid(0x80000000);
        info.max_ext_cpuid = max_extended;
        
        info.vendor[0..4].copy_from_slice(&ebx.to_le_bytes());
        info.vendor[4..8].copy_from_slice(&edx.to_le_bytes());
        info.vendor[8..12].copy_from_slice(&ecx.to_le_bytes());
        info.vendor_len = 12;
        
        if max_extended >= 0x80000008 {
            let (_, _, ecx, _) = cpuid(0x80000008);
            info.cores = ((ecx & 0xFF) + 1) as u32;
            info.threads = info.cores;
        }
        
        if max_extended >= 0x80000004 {
            for i in 0..3 {
                let leaf = 0x80000002 + i as u32;
                let (a, b, c, d) = cpuid(leaf);
                
                let offset = (i * 16) as usize;
                info.brand[offset..offset+4].copy_from_slice(&a.to_le_bytes());
                info.brand[offset+4..offset+8].copy_from_slice(&b.to_le_bytes());
                info.brand[offset+8..offset+12].copy_from_slice(&c.to_le_bytes());
                info.brand[offset+12..offset+16].copy_from_slice(&d.to_le_bytes());
                info.brand_len = (i + 1) * 16;
            }
            
            while info.brand_len > 0 && info.brand[info.brand_len - 1] == 0 {
                info.brand_len -= 1;
            }
        }
        
        let (_, _, ecx, edx) = cpuid(1);
        let mut feature_idx = 0;
        
        if edx & (1 << 23) != 0 { info.features[feature_idx] = "MMX"; feature_idx += 1; }
        if edx & (1 << 25) != 0 { info.features[feature_idx] = "SSE"; feature_idx += 1; }
        if edx & (1 << 26) != 0 { info.features[feature_idx] = "SSE2"; feature_idx += 1; }
        if ecx & (1 << 0)  != 0 { info.features[feature_idx] = "SSE3"; feature_idx += 1; }
        if ecx & (1 << 9)  != 0 { info.features[feature_idx] = "SSSE3"; feature_idx += 1; }
        if ecx & (1 << 19) != 0 { info.features[feature_idx] = "SSE4.1"; feature_idx += 1; }
        if ecx & (1 << 20) != 0 { info.features[feature_idx] = "SSE4.2"; feature_idx += 1; }
        if ecx & (1 << 25) != 0 { info.features[feature_idx] = "AES"; feature_idx += 1; }
        if ecx & (1 << 28) != 0 { info.features[feature_idx] = "AVX"; feature_idx += 1; }
        
        let (max_extended, _, _, _) = cpuid(0x80000000);
        if max_extended >= 0x80000001 {
            let (_, _, ecx, edx) = cpuid(0x80000001);
            if edx & (1 << 29) != 0 { info.features[feature_idx] = "LM"; feature_idx += 1; }
            if ecx & (1 << 5)  != 0 { info.features[feature_idx] = "LZCNT"; feature_idx += 1; }
        }
        
        info.features_len = feature_idx;
    }
    
    info
}

#[allow(unused)]
#[allow(asm_sub_register)]
unsafe fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let mut eax = leaf;
    let mut ebx = 0;
    
    let mut ecx = 0;
    let mut edx = 0;
    
    asm!(
        "xchg {tmp}, rbx",
        "cpuid",
        "xchg {tmp}, rbx",
        tmp = inout(reg) ebx,
        inout("eax") eax,
        out("ecx") ecx,
        out("edx") edx,
        options(nomem, nostack, preserves_flags)
    );
    
    (eax, ebx, ecx, edx)
}

fn format_bytes(bytes: &[u8], len: usize) -> &str {
    let len = bytes[..len].iter().position(|&b| b == 0).unwrap_or(len);
    core::str::from_utf8(&bytes[..len]).unwrap_or("<invalid>")
}

pub fn show_cpu_info() {
    let cpu_info = get_cpuid_info();
    
    print!(("\nCPU Information:"), fg: Color::LightBlue);
    
    print!(("\n  Vendor: "), fg: Color::LightBlue);
    let vendor_str = match &cpu_info.vendor[..12] {
        b"AuthenticAMD" => "AMD",
        b"GenuineIntel" => "Intel",
        b"HygonGenuine" => "Hygon",
        b"GenuineTMx86" => "Transmeta",
        b"Geode by NSC" => "NSC",
        b"CentaurHauls" => "VIA",
        b"VIA VIA VIA " => "VIA",
        b"KVMKVMKVM\0\0\0" => "KVM",
        b"Microsoft Hv" => "Microsoft Hyper-V",
        b"VMwareVMware" => "VMware",
        b"XenVMMXenVMM" => "Xen",
        _ => "Unknown"
    };
    print!(("{}", vendor_str), fg: Color::White);
    
    if cpu_info.brand_len > 0 {
        print!(("\n  Brand:  "), fg: Color::LightBlue);
        let brand_str = format_bytes(&cpu_info.brand, cpu_info.brand_len);
        print!(("{}", brand_str.trim()), fg: Color::White);
    }
    
    print!(("\n"));
    
    print!(("\n  Cores: "), fg: Color::LightBlue);
    print!(("{}", cpu_info.cores), fg: Color::White);

    print!(("\n  Threads: "), fg: Color::LightBlue);
    print!(("{}", cpu_info.threads), fg: Color::White);

    print!(("\n"));

    print!(("\n  Family: "), fg: Color::LightBlue);
    print!(("{:#x}", cpu_info.family), fg: Color::White);

    print!(("\n  Model: ",), fg: Color::LightBlue);
    print!(("{:#x}", cpu_info.model), fg: Color::White);

    print!(("\n  Stepping: "), fg: Color::LightBlue);
    print!(("{}", cpu_info.stepping), fg: Color::White);
    
    print!(("\n"));

    print!(("\n  CPUID:  "), fg: Color::LightBlue);
    
    print!(("\n    Basic: "), fg: Color::LightBlue);
    print!(("{:#x}", cpu_info.max_cpuid), fg: Color::White);
    
    print!(("\n    Extended: "), fg: Color::LightBlue);
    print!(("{:#x}", cpu_info.max_ext_cpuid), fg: Color::White);

    print!(("\n"));

    if vendor_str == "KVM" || vendor_str == "VMware" || vendor_str == "Microsoft Hyper-V" || vendor_str == "Xen" {
        print!(("\n  \u{1f4bb} Running in ",), fg: Color::Yellow);
        print!(("{}", vendor_str), fg: Color::Yellow);

        print!(("\n"));
    }
    
    if cpu_info.features_len > 0 {
        print!(("\n  Features: "), fg: Color::LightBlue);
        
        for i in 0..cpu_info.features_len {
            if i > 0 {
                print!((", "), fg: Color::LightGray);
            }
            print!(("{}", cpu_info.features[i]), fg: Color::Green);
        }
    }
    
}

pub fn handle_cpu_command(args: &[&str]) {
    if args.is_empty() {
        show_cpu_info();
        print!(("\n"));
        print!(("\nFor help use cpu --help"), fg: Color::LightGray);
        return;
    }

    match args[0] {
        "info" => show_cpu_info(),
        "set" => handle_set_command(&args[1..]),
        "halt" => halt_cpu(),
        "--help" | "-h" | "help" => print_help(),
        _ => print!(("\nUnknown CPU command. Try 'cpu --help' for usage."), fg: Color::Red),
    }
}

fn handle_set_command(_args: &[&str]) {
    print!(("\nError: CPU register setting is not implemented yet"), fg: Color::Red);
}

fn halt_cpu() {
    print!(("\nHalting CPU..."), fg: Color::Yellow);
    unsafe {
        loop {
            asm!("hlt", options(nomem, preserves_flags));
        }
    }
}

fn print_help() {
    print!(("\nCPU Commands:"), fg: Color::LightBlue);
    print!(("\n  cpu info       - Show CPU information"), fg: Color::White);
    print!(("\n  cpu set        - Set CPU register (not implemented)"), fg: Color::White);
    print!(("\n  cpu halt      - Halt the CPU"), fg: Color::White);
    print!(("\n  cpu --help    - Show this help"), fg: Color::White);
}
