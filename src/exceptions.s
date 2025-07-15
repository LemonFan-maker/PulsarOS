/* src/exceptions.s */
.section ".text.boot"
.global _exception_vectors
 
// 确保向量表2048字节对齐
.balign 2048 
_exception_vectors:
    // Exception from Current EL with SP_ELx
    b    current_elx_sync     // Synchronous
    b    current_elx_irq      // IRQ (This is our target)
    b    unhandled_exception  // FIQ / SError
    b    unhandled_exception  // Error
 
    // Jump table padding to cover all 16 entries
    .rept 12
        b    unhandled_exception
    .endr
 
// Our main IRQ handler entry point
.global current_elx_irq
current_elx_irq:
    // 1. Save context (all general-purpose registers and the link register)
    sub  sp, sp, #256         // Allocate space on the stack (32 regs * 8 bytes)
    stp  x0,  x1,  [sp, #16 * 0]
    stp  x2,  x3,  [sp, #16 * 1]
    stp  x4,  x5,  [sp, #16 * 2]
    stp  x6,  x7,  [sp, #16 * 3]
    stp  x8,  x9,  [sp, #16 * 4]
    stp  x10, x11, [sp, #16 * 5]
    stp  x12, x13, [sp, #16 * 6]
    stp  x14, x15, [sp, #16 * 7]
    stp  x16, x17, [sp, #16 * 8]
    stp  x18, x19, [sp, #16 * 9]
    stp  x20, x21, [sp, #16 * 10]
    stp  x22, x23, [sp, #16 * 11]
    stp  x24, x25, [sp, #16 * 12]
    stp  x26, x27, [sp, #16 * 13]
    stp  x28, x29, [sp, #16 * 14]
    str  x30,     [sp, #16 * 15] // Save Link Register (x30)
 
    // 2. Call the high-level Rust handler
    bl   handle_irq
 
    // 3. Restore context
    ldr  x30,     [sp, #16 * 15]
    ldp  x28, x29, [sp, #16 * 14]
    ldp  x26, x27, [sp, #16 * 13]
    ldp  x24, x25, [sp, #16 * 12]
    ldp  x22, x23, [sp, #16 * 11]
    ldp  x20, x21, [sp, #16 * 10]
    ldp  x18, x19, [sp, #16 * 9]
    ldp  x16, x17, [sp, #16 * 8]
    ldp  x14, x15, [sp, #16 * 7]
    ldp  x12, x13, [sp, #16 * 6]
    ldp  x10, x11, [sp, #16 * 5]
    ldp  x8,  x9,  [sp, #16 * 4]
    ldp  x6,  x7,  [sp, #16 * 3]
    ldp  x4,  x5,  [sp, #16 * 2]
    ldp  x2,  x3,  [sp, #16 * 1]
    ldp  x0,  x1,  [sp, #16 * 0]
    add  sp, sp, #256
 
    // 4. Return from exception
    eret
 
 
// A catch-all handler for unexpected exceptions
.global unhandled_exception
.global current_elx_sync
unhandled_exception:
current_elx_sync:
    // Simple loop for now. In a real OS, this would print debug info.
    b .