# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

OBJD = $(TOOLCHAIN)objdump
OFLAGS = --disassemble-all --demangle --no-show-raw-insn --wide -M intel
OFLAGS_FULL = $(OFLAGS) --source

BOCHS = bochs -f cfg/bochs.cfg -q

GDB_PORT = 1234
QFLAGS_GDB = $(QFLAGS) -S -gdb tcp::$(GDB_PORT)
GDB = gdb-multiarch
GDB_OPTS = -ex 'target remote :$(GDB_PORT)'

DISASOBJS = $(notdir $(AOBJ) $(KERNBIN))
DISAS = $(DISASOBJS:%=$(DISASDIR)/%.txt)

ADDR2LINE = $(TOOLCHAIN)addr2line
ADDR2LINE_FLAGS = --addresses --inlines --pretty-print --functions --demangle

disassembly: $(DISAS)

bochs: $(KERNISO)
	@$(call ECHO, bochs, $(<F))
	@$(BOCHS)

gdb: $(KERNISO)
	@$(call ECHO, qemu, $(<F))
	@$(QEMU) $(QFLAGS_GDB) -cdrom $< &
	@$(call ECHO, gdb, $(KERNBIN))
	@$(GDB) $(GDB_OPTS) $(KERNBIN)

$(DISASDIR)/%.txt: $(OBJDIR)/% $(DISASDIR)
	@$(call ECHO, objd)
	@$(OBJD) $(OFLAGS) $< > $@

$(DISASDIR)/%.txt: $(BUILDDIR)/% $(DISASDIR)
	@$(call ECHO, objd)
	@$(OBJD) $(OFLAGS) $< > $@

LOG ?= qemu.log

symbolize:
	@cat $(LOG) | grep -A 100 Backtrace: | tail -n +2 | awk '{print $$2}' | xargs \
	        $(ADDR2LINE) $(ADDR2LINE_FLAGS) -e $(KERNBIN) | awk '{print NR ") " $$0}'

.PHONY: disassembly bochs gdb symbolize
