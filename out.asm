bits 64
section .text
global main
extern printf
extern exit

section .data
fibdp:
    dq 0

section .text
.text
.global fib
fib:
	push rbp
	mov rbp, rsp
	sub rsp, 64
	push r12
	push r13
	push r14
	push r15
	mov dword [rbp+4], r10d
	lea r10, [rbp+4]
	movsxd r10, dword [r10]
	mov r11, 0
	cmp r10, r11
	sete r10b
	movzx r10, r10b
	cmp r10, 0
	je .L3
	mov r10, 1
	jmp .L4
.L3:
	lea r11, [rbp+4]
	movsxd r11, dword [r11]
	mov rbx, 1
	cmp r11, rbx
	sete r11b
	movzx r11, r11b
	mov r10, r11
	cmp r10, 0
	je .L4
	mov r10, 1
.L4:
	cmp r10, 0
	je .L1
	lea r10, [rbp+4]
	movsxd r10, dword [r10]
	mov rax, r10
	jmp .Lend0
	jmp .L2
.L1:
	lea r10, fibdp
	lea r11, [rbp+4]
	movsxd r11, dword [r11]
	mov rbx, 4
	mov rax, rbx
	mul r11
	mov r11, rax
	add r10, r11
	movsxd r10, dword [r10]
	mov r11, 0
	cmp r10, r11
	setne r10b
	movzx r10, r10b
	cmp r10, 0
	je .L5
	lea r10, fibdp
	lea r11, [rbp+4]
	movsxd r11, dword [r11]
	mov rbx, 4
	mov rax, rbx
	mul r11
	mov r11, rax
	add r10, r11
	movsxd r10, dword [r10]
	mov rax, r10
	jmp .Lend0
	jmp .L6
.L5:
	lea r10, [rbp+4]
	movsxd r10, dword [r10]
	mov r11, 2
	sub r10, r11
	mov rdi, r10
	push r10
	push r11
	mov rax, 0
	call fib
	pop r11
	pop r10
	mov r11, rax
	lea r10, [rbp+4]
	movsxd r10, dword [r10]
	mov rbx, 1
	sub r10, rbx
	mov rdi, r10
	push r10
	push r11
	mov rax, 0
	call fib
	pop r11
	pop r10
	mov rbx, rax
	add r11, rbx
	lea r10, fibdp
	lea rbx, [rbp+4]
	movsxd rbx, dword [rbx]
	mov r12, 4
	mov rax, r12
	mul rbx
	mov rbx, rax
	add r10, rbx
	mov dword [r10], r11d
	lea r10, fibdp
	lea r11, [rbp+4]
	movsxd r11, dword [r11]
	mov rbx, 4
	mov rax, rbx
	mul r11
	mov r11, rax
	add r10, r11
	movsxd r10, dword [r10]
	mov rax, r10
	jmp .Lend0
.L6:
.L2:
.Lend0:
	pop r15
	pop r14
	pop r13
	pop r12
	mov rsp, rbp
	pop rbp
	ret
.text
.global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 64
	push r12
	push r13
	push r14
	push r15
	mov r10, 0
	lea r11, [rbp+4]
	mov dword [r11], r10d
.L7:
	lea r10, [rbp+4]
	movsxd r10, dword [r10]
	mov r11, 100
	cmp r10, r11
	setl r10b
	movzx r10, r10b
	cmp r10, 0
	je .L8
	mov r10, 0
	lea r11, fibdp
	lea rbx, [rbp+4]
	movsxd rbx, dword [rbx]
	mov r12, 4
	mov rax, r12
	mul rbx
	mov rbx, rax
	add r11, rbx
	mov dword [r11], r10d
	lea r10, [rbp+4]
	movsxd r11, dword [r10]
	add r11, 1
	mov dword [r10], r11d
	sub r11, 1
	jmp .L7
.L8:
.L9:
	mov r10, 46
	mov rdi, r10
	push r10
	push r11
	mov rax, 0
	call fib
	pop r11
	pop r10
	mov r11, rax
	lea r10, [rbp+8]
	mov dword [r10], r11d
	lea r10, [rbp+8]
	movsxd r10, dword [r10]
	mov rax, r10
	jmp .Lend1
.Lend1:
	pop r15
	pop r14
	pop r13
	pop r12
	mov rsp, rbp
	pop rbp
	ret
