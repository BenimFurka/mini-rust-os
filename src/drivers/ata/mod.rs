use crate::port::{inb, outb};
use core::fmt;

const ATA_PRIMARY: u16 = 0x1F0;
const ATA_SECONDARY: u16 = 0x170;

#[derive(Debug, Clone, Copy)]
pub enum AtaDevice {
    Primary,
    Secondary,
    PrimarySlave,
    SecondarySlave,
}

#[derive(Debug)]
#[allow(unused)]
pub enum AtaError {
    DeviceFault(u8),
    DeviceNotReady(u8),
    DataRequestFailed(u8),
    Timeout,
    InvalidSector,
    NoDevice,
}

impl fmt::Display for AtaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtaError::DeviceFault(status) => write!(f, "Device fault (status: 0x{:02X})", status),
            AtaError::DeviceNotReady(status) => write!(f, "Device not ready (status: 0x{:02X})", status),
            AtaError::DataRequestFailed(status) => write!(f, "Data request failed (status: 0x{:02X})", status),
            AtaError::Timeout => write!(f, "Device timeout"),
            AtaError::InvalidSector => write!(f, "Invalid sector"),
            AtaError::NoDevice => write!(f, "No device present"),
        }
    }
}

#[allow(unused)]
pub struct AtaController {
    base: u16,
    ctrl: u16,
    is_slave: bool,
}

impl AtaController {
    pub fn new(device: AtaDevice) -> Self {
        let (base, ctrl, is_slave) = match device {
            AtaDevice::Primary => (ATA_PRIMARY, ATA_PRIMARY + 0x206, false),
            AtaDevice::PrimarySlave => (ATA_PRIMARY, ATA_PRIMARY + 0x206, true),
            AtaDevice::Secondary => (ATA_SECONDARY, ATA_SECONDARY + 0x206, false),
            AtaDevice::SecondarySlave => (ATA_SECONDARY, ATA_SECONDARY + 0x206, true),
        };
        
        AtaController { base, ctrl, is_slave }
    }
    
    pub fn identify(&self) -> Result<[u16; 256], AtaError> {
        let mut buffer = [0u16; 256];
        
        unsafe {
            let drive_select = if self.is_slave { 0xB0 } else { 0xA0 };
            outb(self.base + 6, drive_select);
            
            self.wait_ready()?;
            
            outb(self.base + 7, 0xEC);
            
            for _ in 0..1000 { core::arch::asm!("pause"); }
            
            let status = inb(self.base + 7);
            if status == 0 || (status & 0x80) != 0 {
                return Err(AtaError::NoDevice);
            }
            
            self.wait_data()?;
            
            for i in 0..256 {
                let low = inb(self.base) as u16;
                let high = inb(self.base + 1) as u16;
                buffer[i] = low | (high << 8);
            }
            
            if buffer[0] == 0 {
                return Err(AtaError::NoDevice);
            }
        }
        
        Ok(buffer)
    }
    
    pub fn read_sectors(&self, lba: u32, count: u8, buffer: &mut [u8]) -> Result<(), AtaError> {
        self.wait_ready()?;
        
        unsafe {
            outb(self.base + 6, 0x40 | (if self.is_slave { 0x10 } else { 0 }) | ((lba >> 24) as u8 & 0x0F));
            outb(self.base + 2, count);
            outb(self.base + 3, lba as u8);
            outb(self.base + 4, (lba >> 8) as u8);
            outb(self.base + 5, (lba >> 16) as u8);
            
            outb(self.base + 7, 0x20);
            
            let sector_size = 512;
            
            for i in 0..count {
                self.wait_data()?;
                
                let offset = i as usize * sector_size;
                for j in 0..(sector_size / 2) {
                    let data = inb(self.base) as u16 | ((inb(self.base + 1) as u16) << 8);
                    buffer[offset + j*2] = data as u8;
                    buffer[offset + j*2 + 1] = (data >> 8) as u8;
                }
            }
        }
        
        Ok(())
    }
    
    fn status(&self) -> u8 {
        unsafe { inb(self.base + 7) }
    }
    
    fn wait_ready(&self) -> Result<(), AtaError> {
        for _ in 0..100000 {
            let status = self.status();
            
            if (status & 0x80) == 0 {
                if (status & 0x40) != 0 {
                    return Ok(());
                }
            }
            
            if (status & 0x21) != 0 {
                return Err(AtaError::DeviceFault(status));
            }
            
            for _ in 0..1000 { unsafe { core::arch::asm!("pause"); } }
        }
        Err(AtaError::Timeout)
    }
    
    fn wait_data(&self) -> Result<(), AtaError> {
        for _ in 0..100000 {
            let status = self.status();
            
            if (status & 0x80) != 0 {
                continue;
            }
            
            if (status & 0x21) != 0 {
                return Err(AtaError::DeviceFault(status));
            }
            
            if (status & 0x08) != 0 {
                return Ok(());
            }
            
            for _ in 0..1000 { unsafe { core::arch::asm!("pause"); } }
        }
        Err(AtaError::Timeout)
    }
}
