# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

ARCH ?= x64
TRIPLE ?= x86_64-elf-
TOOLCHAIN = toolchain/bin/$(TRIPLE)

BUILDDIR = build
OBJDIR = $(BUILDDIR)/obj
ISODIR = $(BUILDDIR)/iso
RUSTDIR = $(BUILDDIR)/rust
RBUILDDIR = $(RUSTDIR)/target/debug
DISASDIR = $(BUILDDIR)/disas

LN = ln -sf

AS = nasm
AFLAGS = -f elf64

CARGO = cargo
CFLAGS = --target kernel/arch/$(ARCH)/target.json

LD = $(TOOLCHAIN)ld
LFLAGS = -T kernel/arch/$(ARCH)/link.ld \
         -Map $(BUILDDIR)/map.txt \
         -z max-page-size=0x1000 \
         --gc-sections

ISO = grub-mkrescue
IFLAGS = -follow-links -no-pad
GRUB_CFG = grub.cfg

QEMU = qemu-system-x86_64
QFLAGS = -m 1G -serial stdio

RKERNLIB = $(shell pwd)/$(RBUILDDIR)/libkernel.a
KERNLIB = $(BUILDDIR)/libkernel.a

ASRC = start.s
OBJS = $(ASRC:%.s=$(OBJDIR)/%.o) $(KERNLIB)
KERNBIN = $(BUILDDIR)/kernel.bin
KERNISO = $(BUILDDIR)/ree.iso

ECHO = printf '%5s %s\n\c' $1 $2 $(@F)

all: qemu

qemu: $(KERNISO)
	@$(call ECHO, qemu, $(<F))
	@$(QEMU) $(QFLAGS) -cdrom $<

clippy:
	@$(call ECHO, cargo)
	@$(CARGO) clippy $(CFLAGS) -- -W clippy::all

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

$(KERNLIB): $(RKERNLIB)
	@$(LN) $(realpath $<) $@

$(RKERNLIB): $(RUSTDIR)
	@$(call ECHO, cargo)
	@$(CARGO) build $(CFLAGS)

$(OBJDIR) $(RUSTDIR) $(DISASDIR):
	@mkdir -p $@

$(ISODIR):
	@mkdir -p $@/boot/grub

clean:
	@$(call ECHO)
	@rm -rf $(BUILDDIR) $(KERNBIN) $(KERNISO)

-include debug.mk
-include toolchain.mk
-include $(RBUILDDIR)/libkernel.d

.PHONY: all qemu clippy clean
.DELETE_ON_ERROR:
