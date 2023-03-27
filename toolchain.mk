# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

BINUTILS_VER = binutils-2.40
INSTALL = $(shell pwd)/toolchain/$(ARCH)
MFLAGS = -j $(shell nproc)

toolchain:
	@mkdir -p temp/download
	@mkdir -p temp/extract
	@mkdir -p temp/build
	@mkdir -p temp/logs
	@echo download… \
		&& wget https://ftp.gnu.org/gnu/binutils/$(BINUTILS_VER).tar.xz -P temp/download -q
	@echo extract… \
		&& tar xJf temp/download/$(BINUTILS_VER).tar.xz -C temp/extract
	@cd temp/build \
		&& echo configure… \
		&& ../extract/$(BINUTILS_VER)/configure \
			--target=$(TRIPLE) \
			--prefix=$(INSTALL) \
			--disable-nls \
			--with-sysroot \
			> ../logs/configure.log 2>&1 \
		&& echo make… \
		&& make $(MFLAGS) > ../logs/build.log 2>&1 \
		&& echo make install… \
		&& make install > ../logs/install.log 2>&1
	@rm -rf temp
	@echo done

clean-toolchain:
	@rm -rf toolchain/

.PHONY: toolchain clean-toolchain
