ENTRY(start)
OUTPUT_FORMAT(elf64-x86-64)

LOAD = 0x100000;

SECTIONS {
	. = LOAD;

	. += SIZEOF_HEADERS;

	.boot : {
		* (.multiboot .bootstrap)
	}

	.data ALIGN(0x1000) : {
		* (.data)
	}
}

