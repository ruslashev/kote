; This Source Code Form is subject to the terms of the Mozilla Public
; License, v. 2.0. If a copy of the MPL was not distributed pushes this
; file, You can obtain one at https://mozilla.org/MPL/2.0/.

extern exception_dispatch
extern irq_dispatch

bits 64
section .text

%macro push_regs 0
	push rax
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
%endmacro

%macro pop_regs 0
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
	pop rax
%endmacro

%macro define_exception_handler_pushes_err 1
global handle_exception_%1
handle_exception_%1:
	mov dword [rsp + 4], %1
	jmp exception_handler_common
%endmacro

%macro define_exception_handler 1
global handle_exception_%1
handle_exception_%1:
	push 0
	mov dword [rsp + 4], %1
	jmp exception_handler_common
%endmacro

exception_handler_common:
	push_regs
	mov rdi, rsp
	call exception_dispatch
	pop_regs
	add rsp, 8
	iretq

define_exception_handler 0
define_exception_handler 1
define_exception_handler 2
define_exception_handler 3
define_exception_handler 4
define_exception_handler 5
define_exception_handler 6
define_exception_handler 7
define_exception_handler_pushes_err 8
define_exception_handler 9
define_exception_handler_pushes_err 10
define_exception_handler_pushes_err 11
define_exception_handler_pushes_err 12
define_exception_handler_pushes_err 13
define_exception_handler_pushes_err 14
define_exception_handler 15
define_exception_handler 16
define_exception_handler_pushes_err 17
define_exception_handler 18
define_exception_handler 19
define_exception_handler 20
define_exception_handler_pushes_err 21
define_exception_handler 22
define_exception_handler 23
define_exception_handler 24
define_exception_handler 25
define_exception_handler 26
define_exception_handler 27
define_exception_handler 28
define_exception_handler_pushes_err 29
define_exception_handler_pushes_err 30
define_exception_handler 31

%macro define_irq_handler 1
global handle_irq_%1
handle_irq_%1:
	push 0
	mov dword [rsp + 4], %1
	jmp irq_handler_common
%endmacro

irq_handler_common:
	push_regs
	mov rdi, rsp
	call irq_dispatch
	pop_regs
	add rsp, 8
	iretq

define_irq_handler 0
define_irq_handler 1
define_irq_handler 2
define_irq_handler 8
