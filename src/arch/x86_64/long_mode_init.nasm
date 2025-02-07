global long_mode_start
global read_keyboard_input

section .text
bits 64

read_keyboard_input:
    in al, 0x60       
    test al, 0x80     
    jnz .key_released
    ret               

.key_released:
    xor al, 0x80
    mov al, 0        
    ret

long_mode_start:
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    extern rust_main
    call rust_main

    hlt