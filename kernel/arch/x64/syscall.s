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

	; Reuse the `RegisterFrame` struct
	push 0             ; SS (can't push SS, CS)
	push rsp           ; user RSP
	add qword [rsp], 8 ; adjust RSP
	push r11           ; RFLAGS
	push 0             ; CS
	push rcx           ; RIP
	push 0             ; vector and error code (unused)
	push rax
	push rbx
	push 0             ; RCX (modified by syscall)
	push rdx
	push rsi
	push rdi
	push rbp
	push r8
	push r9
	push r10
	push 0             ; R11 (modified by syscall)
	push r12
	push r13
	push r14
	push r15

	mov [rsp + 160], ss
	mov [rsp + 136], cs

	; ; Terrible improvised hack, this should be done through gs instead
	mov rdi, sysc_saved_rsp
	mov [rdi], rsp
	mov rdi, rsp
	mov rsp, sysc_stack_bot

	call syscall_dispatch

	mov rdi, sysc_saved_rsp
	mov rsp, [rdi]

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
	add rsp, 8 ; RAX (unused)
	add rsp, 8 ; vector and error code
	pop rcx    ; RIP
	add rsp, 8 ; CS
	pop r11    ; RFLAGS
	pop rsp    ; user RSP
	add rsp, 8 ; SS
	sub rsp, 8 ; adjust RSP

	o64 sysret

