# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

MAKE_CFG := $(shell ./config.rs --make > config.mk)
CARGO_CFG := $(shell ./config.rs --cargo)
include config.mk

ifeq ($(CFG_ARCH), x64)
    TRIPLE = x86_64-elf

    AS = nasm
    AFLAGS = -f elf64

    ifndef RELEASE
        AFLAGS += -g
    endif

    ASRC = start.s interrupts.s syscall.s

    QEMU = qemu-system-x86_64
    QFLAGS = -m 5G
else ifeq ($(CFG_ARCH), aarch64)
    TRIPLE = aarch64-elf

    QEMU = qemu-system-aarch64
    QFLAGS = -m 1G \
             -machine virt \
             -cpu cortex-a57
else
    $(error Unknown architecture "$(CFG_ARCH)")
endif

TOOLCHAIN = toolchain/$(CFG_ARCH)/bin/$(TRIPLE)-

BUILDDIR = $(shell pwd)/build
OBJDIR = $(BUILDDIR)/obj
ISODIR = $(BUILDDIR)/iso
RUSTDIR = $(BUILDDIR)/rust
RBUILDDIR = $(RUSTDIR)/*-kernel/*
BUNDLEDIR = $(BUILDDIR)/bundle
DISASDIR = $(BUILDDIR)/disas

LN = ln -sf

CARGO = cargo
CFLAGS = --target kernel/arch/$(CFG_ARCH)/$(CFG_ARCH)-kernel.json

LD = $(TOOLCHAIN)ld
LFLAGS = -T kernel/arch/$(CFG_ARCH)/link.ld \
         -z max-page-size=0x1000 \
         -z noexecstack \
         --gc-sections

ISO = grub-mkrescue
IFLAGS = -follow-links -no-pad

GRUB_CFG = cfg/grub.cfg

QFLAGS += -chardev stdio,id=serial0,logfile=qemu.log \
          -serial chardev:serial0 \
          -no-reboot \
          -no-shutdown

ifeq ($(CFG_GRAPHIC), false)
    QFLAGS += -display none
endif

ifdef RELEASE
    CFLAGS += --release
endif

KERNLIB = $(RBUILDDIR)/libkernel.a
USERSPACE_CRATES = $(notdir $(wildcard userspace/*))
USER_CRATES_BINS = $(filter-out ulib, $(USERSPACE_CRATES))
USER_BINS_TARGET = $(USER_CRATES_BINS:%=$(RBUILDDIR)/%)
USERSPACE_BUNDLE = $(USER_BINS_TARGET:$(RBUILDDIR)/%=$(BUNDLEDIR)/%)

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

$(OBJDIR)/%.o: kernel/arch/$(CFG_ARCH)/%.s | $(OBJDIR)
	@$(call ECHO, as)
	@$(AS) $(AFLAGS) $^ -o $@

$(KERNISO): $(KERNBIN) | $(ISODIR)
	@$(call ECHO, iso)
	@$(LN) $(realpath $(KERNBIN)) $(ISODIR)
	@$(LN) $(realpath $(GRUB_CFG)) $(ISODIR)/boot/grub
	@$(ISO) $(IFLAGS) $(ISODIR) -o $@ 2> /dev/null

$(KERNLIB): $(USERSPACE_BUNDLE) config.rs | $(RUSTDIR)
	@$(call ECHO, cargo)
	@$(CARGO_CFG) $(CARGO) build $(CFLAGS) -p kernel

$(BUNDLEDIR)/%: $(RBUILDDIR)/% | $(BUNDLEDIR)
	@$(LN) $^ $@

$(USER_BINS_TARGET): | $(RUSTDIR)
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
-include $(USER_BINS_TARGET:%=%.d)

.PHONY: all iso kernel qemu clippy clean
.NOTPARALLEL:
.DELETE_ON_ERROR:
.SUFFIXES:
MAKEFLAGS += --no-builtin-rules \
             --no-builtin-variables
