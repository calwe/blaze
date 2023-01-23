.global _switch_stack
_switch_stack:
    // RDI: new stack pointer
    // RSI: new entry
    // TODO: save old stack pointer
    xchg bx, bx
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
    mov rsp, rdi
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    ret