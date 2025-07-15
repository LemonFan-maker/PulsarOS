ENTRY(_start)
 
SECTIONS
{
    . = 0x0a000000;
 
    .text : {
        /* _start 必须是第一条可执行指令 */
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata : {
        . = ALIGN(2048); /* 2048字节对齐 */
        KEEP(*(.text.boot)) /* 把包含向量表的段放在这里 */
        . = ALIGN(8); /* 8字节对齐其余的rodata部分 */
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        _bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        _bss_end = .;
    }

    _kernel_end = .;
 
    /* 预留 64KB 的栈空间 */
    . = . + 0x10000; /* 64KB */
    _stack_top = .; /* 定义符号 _stack_top */
}
