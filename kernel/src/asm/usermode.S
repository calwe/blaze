.text
.global _usermode_jump
_usermode_jump:
    mov     ax,     (8 * 8) | 3
    mov     ds,     ax
    mov     es,     ax
    mov     fs,     ax
    mov     gs,     ax
    push    (8 * 8) | 3
    push    rsi                     # stack
    pushf
    push    (7 * 8) | 3
    xchg bx, bx
    push    rdi
    iretq