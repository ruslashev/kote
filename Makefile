# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

ARCH ?= x64
TRIPLE ?= x86_64-elf-
TOOLCHAIN = toolchain/bin/$(TRIPLE)

BUILDDIR = $(shell pwd)/build
OBJDIR = $(BUILDDIR)/obj
ISODIR = $(BUILDDIR)/iso
RUSTDIR = $(BUILDDIR)/rust
RBUILDDIR = $(RUSTDIR)/target/*
BUNDLEDIR = $(BUILDDIR)/bundle
DISASDIR = $(BUILDDIR)/disas

LN = ln -sf

AS = nasm
AFLAGS = -f elf64

CARGO = cargo
CFLAGS = --target kernel/arch/$(ARCH)/target.json

LD = $(TOOLCHAIN)ld
LFLAGS = -T kernel/arch/$(ARCH)/link.ld \
         -z max-page-size=0x1000 \
         --gc-sections

ISO = grub-mkrescue
IFLAGS = -follow-links -no-pad
GRUB_CFG = cfg/grub.cfg

QEMU = qemu-system-x86_64
QFLAGS = -m 5G \
         -chardev stdio,id=serial0,logfile=qemu.log \
         -serial chardev:serial0 \
         -no-reboot \
         -no-shutdown

ifdef RELEASE
	CFLAGS += --release
endif

KERNLIB = $(RBUILDDIR)/libkernel.a
USERSPACE_CRATES = $(notdir $(wildcard userspace/*))
USER_CRATES_BINS = $(USERSPACE_CRATES:%=$(RBUILDDIR)/%)
USERSPACE_BUNDLE = $(USER_CRATES_BINS:$(RBUILDDIR)/%=$(BUNDLEDIR)/%)

ASRC = start.s interrupts.s
AOBJ = $(ASRC:%.s=$(OBJDIR)/%.o)
OBJS = $(AOBJ) $(KERNLIB)
KERNBIN = $(BUILDDIR)/kernel.bin
KERNISO = $(BUILDDIR)/kote.iso

ECHO = printf '%5s %s\n\c' $1 $2 $(@F)

all: qemu

iso: $(KERNISO)

kernel: $(KERNBIN)

qemu: $(KERNISO)
	@$(call ECHO, qemu, $(<F))
	@$(QEMU) $(QFLAGS) -cdrom $^

clippy:
	@$(call ECHO, cargo)
	@$(CARGO) clippy $(CFLAGS) -- -W clippy::all

$(KERNBIN): $(OBJS)
	@$(call ECHO, ld)
	@$(LD) $(LFLAGS) $^ -o $@

$(OBJDIR)/%.o: kernel/arch/$(ARCH)/%.s | $(OBJDIR)
	@$(call ECHO, as)
	@$(AS) $(AFLAGS) $^ -o $@

$(KERNISO): $(KERNBIN) | $(ISODIR)
	@$(call ECHO, iso)
	@$(LN) $(realpath $(KERNBIN)) $(ISODIR)
	@$(LN) $(realpath $(GRUB_CFG)) $(ISODIR)/boot/grub
	@$(ISO) $(IFLAGS) $(ISODIR) -o $@ 2> /dev/null

$(KERNLIB): $(USERSPACE_BUNDLE) | $(RUSTDIR)
	@$(call ECHO, cargo)
	@$(CARGO) build $(CFLAGS) -p kernel

$(BUNDLEDIR)/%: $(RBUILDDIR)/% | $(BUNDLEDIR)
	@$(LN) $^ $@

$(USER_CRATES_BINS): | $(RUSTDIR)
	@$(call ECHO, cargo)
	@$(CARGO) build $(CFLAGS) -p $(@F)

$(OBJDIR) $(RUSTDIR) $(DISASDIR) $(BUNDLEDIR):
	@mkdir -p $@

$(ISODIR):
	@mkdir -p $@/boot/grub

clean:
	@$(call ECHO)
	@rm -rf $(BUILDDIR) $(KERNBIN) $(KERNISO)

-include debug.mk
-include overrides.mk
-include toolchain.mk
-include $(RBUILDDIR)/libkernel.d
-include $(USER_CRATES_BINS:%=%.d)

.PHONY: all iso kernel qemu clippy clean
.DELETE_ON_ERROR:
.SUFFIXES:
