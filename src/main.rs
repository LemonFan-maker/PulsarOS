// src/main.rs

// 1. 基础属性声明
#![no_std]
#![no_main]
#![feature(asm_const)]

// 2. 引入所有需要的模块和类型
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicUsize, Ordering};

// 引入 aarch64-cpu 的正确模块
use aarch64_cpu::asm::barrier; // for isb, dsb, etc.
use aarch64_cpu::registers::*;
use core::arch::asm; // for the asm! macro

// =================================================================================================
// 3. Panic 处理函数
// =================================================================================================

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 确保我们有一个可用的 UART
    if unsafe { GLOBAL_UART.load(Ordering::Relaxed) } != 0 {
        println!("\n\n--- KERNEL PANIC ---");
        if let Some(location) = info.location() {
            println!(
                "Panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            );
        } else {
            println!("Panic occurred but location is unknown.");
        }
        if let Some(message) = info.payload().downcast_ref::<&str>() {
            println!("Message: {}", message);
        } else {
             println!("Panic occurred with no message.");
        }
        println!("--------------------");
    }

    // 进入无限循环，停止系统
    loop {}
}

// =================================================================================================
// 4. UART 驱动
// =================================================================================================

/// 定义 Rockchip UART 寄存器的偏移量
mod UartRegister {
    pub const RBR_THR_DLL: usize = 0x00; // Receive Buffer / Transmit Holding / Divisor Latch Low
    pub const IER_DLH: usize = 0x04;     // Interrupt Enable Register / Divisor Latch High
    pub const FCR_IIR: usize = 0x08;     // FIFO Control Register / Interrupt Identification Register
    pub const LCR: usize = 0x0C;         // Line Control Register
    pub const MCR: usize = 0x10;         // Modem Control Register
    pub const LSR: usize = 0x14;         // Line Status Register
    pub const MSR: usize = 0x18;         // Modem Status Register
    pub const SCR: usize = 0x1C;         // Scratch Register
    pub const SRBR: usize = 0x30;        // Shadow Receive Buffer
    pub const SFE: usize = 0x98;         // Shadow FIFO Enable
    pub const SRT: usize = 0x9C;         // Shadow RCVR Trigger
    pub const STET: usize = 0xA0;        // Shadow TX Empty Trigger
    pub const HTX: usize = 0xA4;         // Halt TX
}

// IIR 寄存器的位掩码
const IIR_INT_STATUS: u8 = 0x01;         // 0=中断挂起, 1=无中断
const IIR_INT_ID_MASK: u8 = 0x0E;        // 中断类型掩码 (bits 1-3)
const IIR_RXDATA_AVAILABLE: u8 = 0x04;   // 接收数据可用
const IIR_TIMEOUT: u8 = 0x0C;            // 字符超时 (FIFO模式)

/// Uart 驱动结构体
pub struct Uart {
    base_address: usize,
}

impl Uart {
    /// 创建一个新的 Uart 实例
    pub fn new(base_address: usize) -> Self {
        Self { base_address }
    }
    
    /// 发送单个字节
    fn putc(&mut self, c: u8) {
        let thr = (self.base_address + UartRegister::RBR_THR_DLL) as *mut u8;
        let lsr = (self.base_address + UartRegister::LSR) as *mut u8;

        // 等待发送 FIFO 为空 (THRE bit is 5)
        while unsafe { read_volatile(lsr) } & (1 << 5) == 0 {
            core::hint::spin_loop();
        }
        unsafe { write_volatile(thr, c) }
    }
    
    /// 开启 UART 的接收中断
    pub fn enable_rx_interrupt(&mut self) {
        let ier_addr = (self.base_address + UartRegister::IER_DLH) as *mut u8;
        unsafe {
            // 读取当前值，然后设置第0位 (Enable Received Data Available Interrupt)
            // 和第2位 (Enable Receiver Line Status Interrupt)
            let current_ier = read_volatile(ier_addr);
            write_volatile(ier_addr, current_ier | 0x01 | 0x04);
            println!("UART RX interrupt enabled: IER={:02x}", read_volatile(ier_addr));
        }
    }

    /// 读取单个字节 (非阻塞)
    pub fn getc(&mut self) -> Option<u8> {
        let lsr = (self.base_address + UartRegister::LSR) as *mut u8;
        let rbr = (self.base_address + UartRegister::RBR_THR_DLL) as *mut u8;
        
        // 检查是否有数据可读 (Data Ready bit is 0)
        if unsafe { read_volatile(lsr) } & 1 != 0 {
            Some(unsafe { read_volatile(rbr) })
        } else {
            None
        }
    }

    pub fn iir(&mut self) -> u8 {
        let iir = (self.base_address + 0x08) as *mut u8; // IIR_FCR is at offset 0x08
        unsafe { read_volatile(iir) }
    }
}

/// 实现 core::fmt::Write trait，让我们可以使用格式化宏
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // 将换行符 \n 转换为 \r\n
            if byte == b'\n' {
                self.putc(b'\r');
            }
            self.putc(byte);
        }
        Ok(())
    }
}


// =================================================================================================
// 5. 全局打印功能
// =================================================================================================

// RK3566 UART2 基地址
const UART2_BASE: usize = 0xFE66_0000;
static GLOBAL_UART: AtomicUsize = AtomicUsize::new(0);

fn init_global_uart() {
    unsafe { GLOBAL_UART.store(UART2_BASE, Ordering::Relaxed) };
}

fn global_uart() -> Uart {
    Uart::new(unsafe { GLOBAL_UART.load(Ordering::Relaxed) })
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    global_uart().write_fmt(args).unwrap();
}

// =================================================================================================
// --> ADD THIS SECTION (6) <--
// 6. GIC (中断控制器)
// =================================================================================================
 
const GICD_BASE: *mut u32 = 0xFD40_0000 as *mut u32;
const UART2_IRQ_ID: usize = 56;
 
// GIC Distributor Control Register
const GICD_CTLR: *mut u32 = GICD_BASE; 
// GICD_CTLR flags
const GICD_CTLR_ENABLE_GROUP1: u32 = 1 << 1;
const GICD_CTLR_ARE_NS: u32 = 1 << 4; // Affinity routing enable for non-secure state
 
fn init_gic() {
    unsafe {
        // --- GIC Distributor (GICD) ---
 
        // 1. 设置中断为 Group 1 (IRQ)
        // GICD_IGROUPRn, a secure-only register. We assume U-Boot has already set this.
 
        // 2. 使能 UART2 的中断 (Set-Enable Register, GICD_ISENABLERn)
        let isenabler1 = GICD_BASE.add(0x104 / 4); // GICD_ISENABLER1 for IRQs 32-63
        write_volatile(isenabler1, 1 << (UART2_IRQ_ID % 32));
        println!("GICD: Enabled UART2 interrupt (ID {})", UART2_IRQ_ID);
 
        // 3. 设置中断优先级 (Interrupt Priority Registers, GICD_IPRIORITYRn)
        // 值越小，优先级越高。0xA0 是一个比较低的(不紧急的)优先级。
        let ipriority_reg = GICD_BASE.add(0x400 / 4).cast::<u8>().add(UART2_IRQ_ID);
        write_volatile(ipriority_reg, 0xA0);
        println!("GICD: Set UART2 interrupt priority");
 
        // 4. 打开 Distributor 的总开关
        let mut ctlr = read_volatile(GICD_CTLR);
        ctlr |= GICD_CTLR_ENABLE_GROUP1 | GICD_CTLR_ARE_NS;
        write_volatile(GICD_CTLR, ctlr);
        barrier::dsb(barrier::SY);
        println!("GICD: Distributor enabled");
 
 
        // --- GIC CPU Interface (配置当前 CPU 核心) ---
 
        // 5. 设置中断优先级屏蔽，允许所有优先级的中断 (0xFF)
        // ICC_PMR_EL1 (Interrupt Priority Mask Register)
        asm!("msr icc_pmr_el1, {0}", in(reg) 0xFFu64);
        barrier::isb(barrier::SY);
        println!("GIC: Set CPU interface priority mask");
 
        // 6. 使能 CPU 的中断分组 (Group 1 a.k.a. IRQ)
        // ICC_IGRPEN1_EL1 (Interrupt Group 1 Enable register)
        asm!("msr icc_igrpen1_el1, {0}", in(reg) 1u64);
        barrier::isb(barrier::SY);
        println!("GIC: Enabled CPU interface Group 1 interrupts");
    }
}
 
 
// =================================================================================================
// --> RENAME THIS TO SECTION (7) <--
// 7. 异常处理
// =================================================================================================

// 声明汇编中定义的向量表
extern "C" {
    static _exception_vectors: u8;
}

/// 初始化异常处理，设置VBAR_EL1
fn init_exceptions() {
    let vector_table_addr = unsafe { &_exception_vectors as *const _ as u64 };
    println!("Setting exception vector table at address 0x{:x}", vector_table_addr);
    VBAR_EL1.set(vector_table_addr);
    
    // 刷新指令缓存，确保 CPU 使用新的向量表
    barrier::isb(barrier::SY);
    println!("Exception vectors initialized");
}

const ICC_IAR1_EL1:u64 = 0; // System register, use asm! to access
const ICC_EOIR1_EL1:u64 = 0;// System register, use asm! to access

/// 高层中断处理函数，由汇编代码调用
#[no_mangle]
pub extern "C" fn handle_irq() {
    let irq_id: u64;
    unsafe { asm!("mrs {0}, icc_iar1_el1", out(reg) irq_id); }
 
    if irq_id == 56 { // UART2
        let _ = global_uart().iir(); // 读取IIR清中断
        if let Some(c) = global_uart().getc() {
            print!("{}", c as char); // 回显
        }
    }
    
    unsafe { asm!("msr icc_eoir1_el1, {0}", in(reg) irq_id); }
}

// =================================================================================================
// 8. PulsarOS 入口点
// =================================================================================================
#[no_mangle]
pub extern "C" fn _start() -> ! {
    extern "C" {
        static _stack_top: u8;
    }
    unsafe {
        asm!(
            "ldr x0, =_stack_top",
            "mov sp, x0",
            options(nostack)
        );
    }
    
    // 初始化UART，让我们能看到输出
    init_global_uart();
 
    // ----------------------------------------------------
    //  !!! 暂时注释掉所有中断和 GIC 相关的初始化 !!!
    // ----------------------------------------------------
    // init_exceptions();
    // init_gic();
    // global_uart().enable_rx_interrupt();
    // unsafe {
    //     barrier::dsb(barrier::SY);
    //     barrier::isb(barrier::SY);
    //     asm!("msr daifclr, #2");
    // }
 
    println!("\n--- PulsarOS Minimal Boot Test ---");
    println!("Stack and UART seem to be working.");
    println!("Now entering infinite loop...");
 
    // 进入一个简单的无限循环，不使用 wfi
        loop {
        // 在循环里可以加点东西，比如一个简单的延时
        for _ in 0..10_000_000 {
            // 把 asm! 调用放到 unsafe 块里
            unsafe {
                asm!("nop");
            }
        }
        print!("."); // 每隔一段时间打印一个点，证明系统在运行
    }
}
// pub extern "C" fn _start() -> ! {
//     // 你的 _start 函数的汇编部分是错误的，我帮你修正一下
//     // 它应该在使用 _stack_top 之前先声明它
//     extern "C" {
//         static _stack_top: u8;
//     }
//     unsafe {
//         asm!(
//             "ldr x0, =_stack_top", // 加载栈顶地址到 x0
//             "mov sp, x0",          // 将 x0 的值赋给 sp
//             options(nostack)       // 告诉编译器这段汇编在设置栈，不要自己用栈
//         );
//     }
    
//     println!("\n\n--- PulsarOS Booting ---");
    
//     // 初始化子系统
//     init_global_uart();
//     println!("UART initialized");
    
//     init_exceptions();
//     init_gic();
    
//     // 启用UART接收中断
//     global_uart().enable_rx_interrupt();
//     println!("UART RX interrupts enabled");
    
//     // 开启全局中断 (清除 PSTATE.I bit)
//     unsafe {
//         barrier::dsb(barrier::SY);
//         barrier::isb(barrier::SY);
//         asm!("msr daifclr, #2"); // Unmask IRQs (Interrupts)
//         println!("Global interrupts enabled");
//     }
 
//     println!("\n--- PulsarOS Booted Successfully ---");
//     println!("Interrupts enabled. Waiting for your input...");
    
//     // 手动测试发送字符串，验证UART发送功能正常工作
//     println!("Hello from PulsarOS! Try typing something...");
 
//     // CPU 进入低功耗等待状态，直到中断发生
//     loop {
//         aarch64_cpu::asm::wfi(); // Wait For Interrupt
//     }
// }
