global start

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

bits 32
section .inittext
start:
	; Clear interrupts
	cli

	; Set up stack
	mov esp, init_stack

	; Save multiboot info
	mov [mb_info], ebx
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
	lgdt [gdt.ptr]

	; Enter long mode
	jmp 8:start64

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
	; PML4[0] -> PDPT
	mov eax, pdpt
	or eax, WRITABLE | PRESENT
	mov [pml4], eax

	; PDPT[0] -> PD
	mov eax, pd
	or eax, WRITABLE | PRESENT
	mov [pdpt], eax

	; PD -> 0x00000000 - 0x00400000
	mov dword [pd + 0], 0x000000 | HUGE | WRITABLE | PRESENT
	mov dword [pd + 8], 0x200000 | HUGE | WRITABLE | PRESENT
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
	mov eax, pml4
	mov cr3, eax

	; Enable paging
	mov eax, cr0
	bts eax, 16 ; Write protect
	bts eax, 31 ; Paging
	mov cr0, eax
	ret

bits 64
start64:
	print "Hello, World!"

hltspin:
	hlt
	jmp hltspin

section .bss
pml4:
	resb 4096
pdpt:
	resb 4096
pd:
	resb 4096
init_stack_bottom:
	resb 4096 * 2
init_stack:

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

