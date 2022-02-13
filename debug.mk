# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

OBJD = $(TOOLCHAIN)objdump
OFLAGS = --disassemble-all --demangle --no-show-raw-insn --wide -M intel
OFLAGS_FULL = $(OFLAGS) --source

DISASOBJS = $(notdir $(OBJS) $(KERNBIN))
DISAS = $(DISASOBJS:%=$(DISASDIR)/%.txt)

disassembly: $(DISAS)

$(DISASDIR)/%.txt: $(OBJDIR)/% $(DISASDIR)
	@$(call ECHO, objd)
	@$(OBJD) $(OFLAGS) $< > $@

$(DISASDIR)/%.txt: $(BUILDDIR)/% $(DISASDIR)
	@$(call ECHO, objd)
	@$(OBJD) $(OFLAGS) $< > $@

.PHONY: disassembly
