; This Source Code Form is subject to the terms of the Mozilla Public
; License, v. 2.0. If a copy of the MPL was not distributed with this
; file, You can obtain one at https://mozilla.org/MPL/2.0/.

global syscall_handler
global sysc_stack_guard_bot

extern sysc_stack_bot
extern sysc_saved_rsp
extern syscall_dispatch

; If, in Rust, pointers could be "cast to integers during const eval", then this could've been
; written as a naked function, akin to `do_switch()`.

syscall_handler:
	; State provided by `syscall` instruction:
	;   rcx = retaddr
	;   r11 = rflags
	;   rsp = user rsp
	;   cs  = kernel CS
	;   ss  = kernel SS
	;
	; Syscall args: rdi rsi rdx r10 (r10 instead of rcx)
	; Syscall num:  rax

	push rsp
	push rbx
	push rcx
	push rdx
	push rsi
	push rdi
	push rbp
	push r8
	push r9
	push r10
	push r11
	push r12
	push r13
	push r14
	push r15

	; Terrible improvised hack, this should be done through gs instead
	mov r11, sysc_saved_rsp
	mov [r11], rsp
	mov rsp, sysc_stack_bot

	push r10
	push rdx
	push rsi
	push rdi
	push rax

	mov rdi, rsp
	call syscall_dispatch

	mov r11, sysc_saved_rsp
	mov rsp, [r11]

	pop r15
	pop r14
	pop r13
	pop r12
	pop r11
	pop r10
	pop r9
	pop r8
	pop rbp
	pop rdi
	pop rsi
	pop rdx
	pop rcx
	pop rbx
	pop rsp

	o64 sysret

