arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.nasm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.nasm, build/arch/$(arch)/%.o, $(assembly_source_files))
target ?= x86_64-unknown-none
rust_os := target/$(target)/debug/libsimple_rust_os.a

.PHONY: all clean run iso

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -vga std -m 512M -cpu qemu64

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

cargo:
	@cargo +nightly build --target $(target) -Zbuild-std=core,alloc

build/arch/$(arch)/%.o: src/arch/$(arch)/%.nasm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
