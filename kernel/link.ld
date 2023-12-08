ENTRY(bootstrap)

SECTIONS {
    . = 8M;

    .bootstrap : {
        . = ALIGN(8);
        target/boot (.text)
    }

    .text : {
        *(EXCLUDE_FILE(target/boot) .text)
    }
    
    .rodata : {
        *(.rodata)
    }

    .data : {
        *(.data)
    }

    .bss : {
        *(COMMON)
        *(.bss)
    }
}
