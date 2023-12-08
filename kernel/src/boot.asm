[bits 32]

PAGEML4SIZE equ 512 * 8
PAGEDIRPTABLESIZE equ 512 * 8
PAGEDIRSIZE equ 512 * 8
PAGETABLESIZE equ 512 * 8
PAGESIZE equ 512 * 4096

BOOT32MAPSIZE equ 512 * PAGESIZE
STACKSIZE equ 1024 * 1024

extern _kmain

section .text
global bootstrap

multiboot2_header:
    incbin "kernel/src/multiboot2_header.bin"

bootstrap:
    mov esp, init_stack_top
    mov [mbinfo], ebx

    ; enable pae
    mov edx, cr4
    or edx, (1 << 5)
    mov cr4, edx

    ; set long mode enable
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    ; PAGING

    ; set first entry in pml4
    mov eax, page_dir_ptable
    or eax, 3 ; RW | Present
    mov [page_ml4], eax
    ; set first entry in pdpt
    mov eax, page_dir
    or eax, 3 ; RW | Present
    mov [page_dir_ptable], eax

    ; identity mapping
    mov ecx, BOOT32MAPSIZE - PAGESIZE ; counter
    mov edi, page_dir + PAGEDIRSIZE - 8 ; page_dir entry addr
identity_map:
    ; set page_table[edi/8] 
    mov eax, ecx
    or eax, 131
    mov [edi], eax
    ; adjust counters and loop
    sub edi, 8
    sub ecx, PAGESIZE
    jnz identity_map
    ; otherwise set final entry
    mov eax, ecx
    or eax, 131
    mov [edi], eax
post_map:
    ; set cr3 to pml4
    mov eax, page_ml4
    mov cr3, eax

    ; enable paging
    or ebx, (1 << 31) | (1 << 0)
    mov cr0, ebx
setgdt:
    ; gdt
    mov eax, 8 * 5 - 1
    mov [gdtr], eax
    mov eax, gdt
    mov [gdtr + 2], eax
    lgdt [gdtr]
    ; segment registers
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; _kmain mbinfo arg
    mov edi, [mbinfo]
    ; switch to 64 bit and jump to _kmain
    jmp 0x08:_kmain

break:
    nop

section .data

gdt dq 0x0                  ; NULL Descriptor
    dq 0x00CF9A000000FFFF   ; Kernel Mode Code Segment
    dq 0x00CF92000000FFFF   ; Kernel Mode Data Segment
    dq 0x00CFFA000000FFFF   ; User Mode Code Segment
    dq 0x00CFF2000000FFFF   ; User Mode Data Segment

gdtr dw 0 ; limit
     dq 0 ; base

mbinfo dw 0

section .bss
align 4096
           
init_stack_bottom:
    resb STACKSIZE   
init_stack_top:

page_dir:
    resb PAGEDIRSIZE

page_table:
    resb PAGETABLESIZE

page_dir_ptable:
    resb PAGEDIRPTABLESIZE

page_ml4:
    resb PAGEML4SIZE
