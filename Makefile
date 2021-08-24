ARCH ?= x64
TRIPLE ?= x86_64-elf-
TOOLCHAIN = toolchain/bin/$(TRIPLE)

BUILDDIR = build
OBJDIR = $(BUILDDIR)/obj
ISODIR = $(BUILDDIR)/iso

LN = ln -sf

AS = nasm
AFLAGS = -f elf64

LD = $(TOOLCHAIN)ld
LFLAGS = -T kernel/arch/$(ARCH)/link.ld \
         -Map $(BUILDDIR)/map.txt \
         -z max-page-size=0x1000

ISO = grub-mkrescue
IFLAGS = -follow-links -no-pad
GRUB_CFG = grub.cfg

QEMU = qemu-system-x86_64
QFLAGS = -m 1G -serial stdio

ASRC = start.s
OBJS = $(ASRC:%.s=$(OBJDIR)/%.o)
KERNBIN = kernel.bin
KERNISO = ree.iso

all: $(KERNISO)

qemu: $(KERNISO)
	$(QEMU) $(QFLAGS) -cdrom $<

$(KERNBIN): $(OBJS)
	$(LD) $(LFLAGS) $< -o $@

$(OBJDIR)/%.o: kernel/arch/$(ARCH)/%.s $(OBJDIR)
	$(AS) $(AFLAGS) $< -o $@

$(KERNISO): $(KERNBIN) $(ISODIR)
	$(LN) $(realpath $(KERNBIN)) $(ISODIR)
	$(LN) $(realpath $(GRUB_CFG)) $(ISODIR)/boot/grub
	$(ISO) $(ISODIR) $(IFLAGS) -o $@ 2> /dev/null

$(OBJDIR):
	@mkdir -p $@

$(ISODIR):
	@mkdir -p $@/boot/grub

clean:
	rm -rf $(BUILDDIR) $(KERNBIN) $(KERNISO)

-include toolchain.mk

.PHONY: all qemu clean
.DELETE_ON_ERROR:
