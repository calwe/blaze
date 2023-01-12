.text
.global _usermode_jump
_usermode_jump:
    mov ax, (8 * 8) | 3
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov rax, 0xdeadbeef1000
    push (8 * 8) | 3
    push rax
    pushf
    push (7 * 8) | 3
    ; lea  rax, [rip + _usermode_function]
    xchg bx, bx
    push rdi
    iretq