/*
 * linker.ld for PulsarOS (Polling Version)
 * This version is simplified and does not handle exception vectors.
 */
 
ENTRY(_start)
 
SECTIONS
{
    /* 内核的起始加载地址 */
    . = 0x0a000000;
 
    /* .text 段 (代码) */
    .text : {
        KEEP(*(.text._start))  /* 确保 _start 函数在最前面 */
        *(.text .text.*)       /* 所有其他的代码 */
    }
 
    /* .rodata 段 (只读数据) */
    .rodata : {
        *(.rodata .rodata.*)
    }
 
    /* .data 段 (已初始化的可写数据) */
    .data : {
        *(.data .data.*)
    }
 
    /* .bss 段 (未初始化的数据) */
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }
 
    /* 预留 64KB 的内核栈 */
    . = ALIGN(16); /* 栈最好是16字节对齐 */
    . = . + 0x10000; /* 64KB stack size */
    _stack_top = .;  /* 栈顶地址，供 _start 函数使用 */
}