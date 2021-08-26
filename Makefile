ARCH ?= x64
TRIPLE ?= x86_64-elf-
TOOLCHAIN = toolchain/bin/$(TRIPLE)

BUILDDIR = build
OBJDIR = $(BUILDDIR)/obj
ISODIR = $(BUILDDIR)/iso
RUSTDIR = $(BUILDDIR)/rust
RBUILDDIR = $(RUSTDIR)/target/debug

LN = ln -sf

AS = nasm
AFLAGS = -f elf64

CARGO = cargo
CFLAGS = -Z build-std=core \
         --target kernel/arch/$(ARCH)/target.json \
         --target-dir $(RUSTDIR)

LD = $(TOOLCHAIN)ld
LFLAGS = -T kernel/arch/$(ARCH)/link.ld \
         -Map $(BUILDDIR)/map.txt \
         -z max-page-size=0x1000

OBJD = $(TOOLCHAIN)objdump
OFLAGS = -D -S -M intel --visualize-jumps --no-show-raw-insn -w

ISO = grub-mkrescue
IFLAGS = -follow-links -no-pad
GRUB_CFG = grub.cfg

QEMU = qemu-system-x86_64
QFLAGS = -m 1G -serial stdio

RKERNLIB = $(RBUILDDIR)/libkernel.a
ASRC = start.s
OBJS = $(ASRC:%.s=$(OBJDIR)/%.o) $(RKERNLIB)
KERNBIN = $(BUILDDIR)/kernel.bin
KERNISO = $(BUILDDIR)/ree.iso

DISAS = $(KERNBIN:%.bin=%.txt)

ECHO = printf '%5s %s\n\c' $1 $2 $(@F)

all: debug qemu

debug: $(DISAS)

qemu: $(KERNISO)
	@$(call ECHO, qemu, $(<F))
	@$(QEMU) $(QFLAGS) -cdrom $<

$(KERNBIN): $(OBJS)
	@$(call ECHO, ld)
	@$(LD) $(LFLAGS) $^ -o $@

$(OBJDIR)/%.o: kernel/arch/$(ARCH)/%.s $(OBJDIR)
	@$(call ECHO, as)
	@$(AS) $(AFLAGS) $< -o $@

$(KERNISO): $(ISODIR) $(KERNBIN)
	@$(call ECHO, iso)
	@$(LN) $(realpath $(KERNBIN)) $(ISODIR)
	@$(LN) $(realpath $(GRUB_CFG)) $(ISODIR)/boot/grub
	@$(ISO) $(IFLAGS) $< -o $@ 2> /dev/null

$(RKERNLIB): $(RUSTDIR)
	@$(call ECHO, cargo)
	@$(CARGO) build $(CFLAGS)

%.txt: %.bin
	@$(call ECHO, objd)
	@$(OBJD) $(OFLAGS) $< > $@

$(OBJDIR):
	@mkdir -p $@

$(ISODIR):
	@mkdir -p $@/boot/grub

$(RUSTDIR):
	@mkdir -p $@

clean:
	@$(call ECHO)
	@rm -rf $(BUILDDIR) $(KERNBIN) $(KERNISO)

-include toolchain.mk

.PHONY: all debug qemu clean
.DELETE_ON_ERROR:
