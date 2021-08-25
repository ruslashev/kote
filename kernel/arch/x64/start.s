global start

section .multiboot
align 8
mb_start:
	%define MB_MAGIC  0xE85250D6
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
section .bootstrap
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

	print "Hello, World!"

hltspin:
	hlt
	jmp hltspin

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
	jmp hltspin

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
	jmp hltspin

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
	jmp hltspin

section .bss
init_stack_bottom:
	resb 4096 * 2
init_stack:

section .data
mb_info:
	dq 0
