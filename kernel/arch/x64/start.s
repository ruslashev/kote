; This Source Code Form is subject to the terms of the Mozilla Public
; License, v. 2.0. If a copy of the MPL was not distributed with this
; file, You can obtain one at https://mozilla.org/MPL/2.0/.

global start
global mb_info
global pml4
global pdpt
global pd

extern kmain
extern __sbss
extern __ebss

%define KERNEL_BASE 0xffffffff80000000
%define KERNEL_STACK_SZ 4096 * 4

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

	; framebuffer tag
	dw 5  ; type
	dw 0  ; flags
	dd 20 ; size
	dd 0  ; width
	dd 0  ; height
	dd 0  ; depth

	dd 0  ; padding for 8-byte alignment

	; end tag
	dw 0 ; type
	dw 0 ; flags
	dd 8 ; size
mb_end:

bits 32
section .inittext
start:
	; Clear interrupt flag
	cli

	; Save eax passed from multiboot
	mov edx, eax

	; Clear .bss
	mov ecx, RELOC(__ebss)
	sub ecx, RELOC(__sbss)
	inc ecx
	xor eax, eax
	mov edi, RELOC(__sbss)
	rep stosb

	; Restore eax
	mov eax, edx

	; Set up stack
	mov esp, RELOC(stack_botmost)

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
%define Addr (1 << 21) ; 2 MiB
%define Huge (1 << 7)
%define WrPr (1 << 1) | (1 << 0) ; Writable and present
	; Identity mapping: 0x0..0x400000 -> 0x0..0x400000; 2 entries, 2 MiB each.
	; pml4[0] -> pdpt
	mov eax, RELOC(pdpt)
	or eax, WrPr
	mov [RELOC(pml4)], eax

	; pdpt[0] -> pd
	mov eax, RELOC(pd)
	or eax, WrPr
	mov [RELOC(pdpt)], eax

	; pd[0], pd[1] -> 0x00000000..0x00400000
	mov dword [RELOC(pd) + 0 * 8], (0 * Addr) | Huge | WrPr
	mov dword [RELOC(pd) + 1 * 8], (1 * Addr) | Huge | WrPr

	; Kernel higher half mapping: 0xffffffff80000000..0xffffffff80400000 -> 0x0..0x400000
	; pml4[511] -> pdpt
	mov eax, RELOC(pdpt)
	or eax, WrPr
	mov [RELOC(pml4) + 511 * 8], eax

	; pdpt[510] -> pd, map at -2 GiB
	mov eax, RELOC(pd)
	or eax, WrPr
	mov [RELOC(pdpt) + 510 * 8], eax
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

	; Set up stack
	mov rsp, stack_botmost
	mov rbp, 0

	; Call rust code
	call kmain

	; In case we return, loop
	cli
hltspin:
	hlt
	jmp hltspin

section .rodata
gdt:
%define RW (1 << 41) ; Readable (for code) / Writable (for data)
%define Ex (1 << 43) ; Executable
%define S  (1 << 44) ; Is a code or data segment
%define K  (0 << 45) ; Kernel privilege level
%define U  (3 << 45) ; User privilege level
%define Pr (1 << 47) ; Present
%define L  (1 << 53) ; Is 64-bit code
	dq 0                        ; null
	dq RW | Ex | S | K | Pr | L ; kernel code
	dq RW      | S | K | Pr     ; kernel data
	dq RW | Ex | S | U | Pr | L ; user code
	dq RW      | S | U | Pr     ; user data
.ptr:
	dw $ - gdt - 1 ; size
	dq gdt         ; offset (address)
.ptr_low:
	dw .ptr - gdt - 1
	dq RELOC(gdt)

section .bss
pml4:
	resb 4096
pdpt:
	resb 4096
pd:
	resb 4096
mb_info:
	resq 1
stack_topmost:
	resb KERNEL_STACK_SZ
stack_botmost:

