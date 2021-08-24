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
	mov ebx, [text_y]
	shl ebx, 1
	jmp %%loop_start
%%strdata: db %1
%%loop_start:
	mov al, [%%strdata + ecx - 1]
	mov [0xb8000 + ecx * 2 + ebx - 2], ax
	loop %%loop_start
	add dword [text_y], 80
%endmacro

section .bootstrap
bits 32
start:
	; Clear interrupts
	cli

	; Save multiboot info
	mov [mb_info], ebx

	; Clear screen
	mov ecx, 2 * 80 * 25
	xor eax, eax
	mov edi, 0xb8000
	rep stosw

	print "Hello, World!"

hltspin:
	hlt
	jmp hltspin

section .data
text_y:
	dd 0
mb_info:
	dq 0
