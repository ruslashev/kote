ARCH ?= x64
TRIPLE ?= x86_64-elf-
TOOLCHAIN = toolchain/bin/$(TRIPLE)

BUILDDIR = build
OBJDIR = $(BUILDDIR)/obj

AS = nasm
AFLAGS = -f elf64

LD = $(TOOLCHAIN)ld
LFLAGS = -T kernel/arch/$(ARCH)/link.ld \
         -Map $(BUILDDIR)/map.txt \
         -z max-page-size=0x1000

ASRC = start.s
OBJS = $(ASRC:%.s=$(OBJDIR)/%.o)
KERNBIN = kernel.bin

all: $(KERNBIN)

$(KERNBIN): $(OBJS)
	$(LD) $(LFLAGS) $< -o $@

$(OBJDIR)/%.o: kernel/arch/$(ARCH)/%.s $(OBJDIR)
	$(AS) $(AFLAGS) $< -o $@

$(OBJDIR):
	@mkdir -p $@

clean:
	rm -rf $(BUILDDIR) $(KERNBIN)

-include toolchain.mk

.PHONY: all clean
.DELETE_ON_ERROR:
