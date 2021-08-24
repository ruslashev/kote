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

section .bootstrap
bits 32
start:
	; Clear interrupts
	cli

	; Clear screen
	xor ecx, ecx
clrscr:
	mov word [0xb8000 + ecx], 0x0000
	add ecx, 2
	cmp ecx, 2 * (80 * 25)
	jnz clrscr

	mov dword [0xb8000], 0x073a0728

hltspin:
	hlt
	jmp hltspin

