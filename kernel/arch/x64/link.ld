ENTRY(start)
OUTPUT_FORMAT(elf64-x86-64)

LOAD = 0x100000;
KERNEL_BASE = 0xffffffff80000000;

SECTIONS {
	. = LOAD;

	. += SIZEOF_HEADERS;

	.init : {
		KEEP (* (.multiboot))
		* (.inittext)
	}

	. += KERNEL_BASE;

	.text ALIGN (0x1000) : AT (ADDR(.text) - KERNEL_BASE) {
		* (.text)
	}

	.data ALIGN (0x1000) : AT (ADDR(.data) - KERNEL_BASE) {
		* (.data)
	}

	.bss  ALIGN (0x1000) : AT (ADDR(.bss)  - KERNEL_BASE) {
		* (.bss)
	}
}

