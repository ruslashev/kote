global start

%define KERNEL_BASE 0xffffffff80000000
%define KERNEL_STACK_SZ 4096 * 2

%define RELOC(x) (x - KERNEL_BASE)

%macro print 1
	mov ecx, %%loop_start - %%strdata
	mov eax, 0x0700
	jmp %%loop_start
%%strdata: db %1
%%loop_start:
	mov al, [%%strdata + ecx - 1]
	mov [0xb8000 + ecx * 2 - 2], ax
	loop %%loop_start
%endmacro

section .multiboot
align 8
mb_start:
	%define MB_MAGIC  0xe85250d6
	%define MB_ARCH   0
	%define MB_LENGTH (mb_end - mb_start)
	%define MB_CHKSUM -(MB_MAGIC + MB_ARCH + MB_LENGTH)

	dd MB_MAGIC
	dd MB_ARCH
	dd MB_LENGTH
	dd MB_CHKSUM

	; end tag
	dw 0
	dw 0
	dd 8
mb_end:

bits 32
section .inittext
start:
	; Clear interrupts
	cli

	; Set up stack
	mov esp, RELOC(init_stack)

	; Save multiboot info
	mov [RELOC(mb_info)], ebx
	push eax

	call clear_screen

	pop eax
	call check_multiboot

	call check_cpuid
	call check_long_mode
	call map_pages
	call setup_long_mode
	call enable_paging

	; Load GDT
	lgdt [RELOC(gdt.ptr_low)]

	; Enter long mode
	jmp 8:start64_low

initspin:
	hlt
	jmp initspin

clear_screen:
	mov ecx, 2 * 80 * 25
	xor eax, eax
	mov edi, 0xb8000
	rep stosw
	ret

check_multiboot:
	cmp eax, 0x36d76289
	jne .fail
	ret
.fail:
	print "Not loaded by a multiboot-compliant bootloader"
	jmp initspin

check_cpuid:
	pushfd
	pop eax          ; eax = flags
	mov ecx, eax     ; copy to compare and restore
	xor eax, 1 << 21 ; flip a bit
	push eax
	popfd            ; write to flags
	pushfd
	pop eax          ; and read from flags
	push ecx
	popfd            ; restore old value
	xor eax, ecx     ; check if the bit was, in fact, flipped
	jz .fail
	ret
.fail:
	print "CPUID instruction unavailable"
	jmp initspin

check_long_mode:
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001 ; check for extended functions
	jbe .fail
	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29   ; long mode bit
	jz .fail
	ret
.fail:
	print "This CPU is not 64-bit capable"
	jmp initspin

map_pages:
%define HUGE     (1 << 7)
%define WRITABLE (1 << 1)
%define PRESENT  (1 << 0)
	; pml4[0] -> pdpt_low (for startup)
	mov eax, RELOC(pdpt_low)
	or eax, WRITABLE | PRESENT
	mov [RELOC(pml4)], eax

	; pml4[511] -> pdpt
	mov eax, RELOC(pdpt)
	or eax, WRITABLE | PRESENT
	mov [RELOC(pml4) + 511 * 8], eax

	; pdpt_low[0] -> pd
	mov eax, RELOC(pd)
	or eax, WRITABLE | PRESENT
	mov [RELOC(pdpt_low)], eax

	; pdpt[510] -> pd, map at -2 GiB, 1 GiB each entry
	mov eax, RELOC(pd)
	or eax, WRITABLE | PRESENT
	mov [RELOC(pdpt) + 510 * 8], eax

	; pd -> 0x00000000 - 0x00400000, 2 entries, 2 MiB each
	mov dword [RELOC(pd) + 0], 0x000000 | HUGE | WRITABLE | PRESENT
	mov dword [RELOC(pd) + 8], 0x200000 | HUGE | WRITABLE | PRESENT
	ret

setup_long_mode:
	; Enable Physical Address Extension
	mov eax, cr4
	bts eax, 5
	mov cr4, eax

	; Enable long mode in EFER
	mov ecx, 0xc0000080
	rdmsr
	bts eax, 0  ; System call extensions
	bts eax, 8  ; Long mode enable
	bts eax, 11 ; No-execute enable
	wrmsr
	ret

enable_paging:
	; Load PML4
	mov eax, RELOC(pml4)
	mov cr3, eax

	; Enable paging
	mov eax, cr0
	bts eax, 16 ; Write protect
	bts eax, 31 ; Paging
	mov cr0, eax
	ret

bits 64
start64_low:
	lgdt [gdt.ptr]

	mov rax, start64
	jmp rax

section .text
start64:
	; Point segment registers to a null GDT entry
	xor ax, ax
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax
	mov ss, ax

	; Clear low-memory page mapping
	xor rax, rax
	mov [pml4], rax

	mov dword [KERNEL_BASE + 0xb8000], 0x073a0728

hltspin:
	hlt
	jmp hltspin

section .data
mb_info:
	dq 0
gdt:
%define RW (1 << 41) ; Readable (for code) / Writable (for data)
%define Ex (1 << 43) ; Executable
%define S  (1 << 44) ; 0 (system) / 1 (user) code and data
%define Pr (1 << 47) ; Present
%define L  (1 << 53) ; Is 64-bit code
	dq 0                    ; null
	dq RW | Ex | S | Pr | L ; code
	dq RW      | S | Pr     ; data
.ptr:
	dw $ - gdt - 1 ; size
	dq gdt         ; offset (address)
.ptr_low:
	dw .ptr - gdt - 1
	dq RELOC(gdt)

section .bss
pml4:
	resb 4096
pdpt_low:
	resb 4096
pdpt:
	resb 4096
pd:
	resb 4096
init_stack_bottom:
	resb KERNEL_STACK_SZ
init_stack:

