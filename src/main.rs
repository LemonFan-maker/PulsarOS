// src/main.rs
 
#![no_std]
#![no_main]

use core::panic::PanicInfo;
 
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


use core::fmt::{self, Write};
use core::ptr::{read_volatile, write_volatile};
 
/// 定义 Rockchip UART 寄存器的偏移量
mod UartRegister {
    pub const RBR_THR_DLL: usize = 0x00; // Receive Buffer/Transmit Holding/Divisor Latch Low
    pub const IER_DLH: usize = 0x04;     // Interrupt Enable/Divisor Latch High
    pub const FCR_IIR: usize = 0x08;     // FIFO Control/Interrupt Identification
    pub const LCR: usize = 0x0C;         // Line Control Register
    pub const MCR: usize = 0x10;         // Modem Control Register
    pub const LSR: usize = 0x14;         // Line Status Register
}
 
/// Uart 驱动结构体
pub struct Uart {
    base_address: usize,
}
 
impl Uart {
    /// 创建一个新的 Uart 实例
    pub fn new(base_address: usize) -> Self {
        Self { base_address }
    }
 
    /// 初始化 UART
    /// U-Boot 已经做过初始化，所以我们通常不需要再次调用。
    /// 但如果需要，可以设置波特率、8N1模式等。
    pub fn init(&mut self) {
        // U-Boot已经设置好了，这里暂时留空
    }
 
    /// 发送单个字节
    fn putc(&mut self, c: u8) {
        let thr = (self.base_address + UartRegister::RBR_THR_DLL) as *mut u8;
        let lsr = (self.base_address + UartRegister::LSR) as *mut u8;
 
        // 等待发送 FIFO 为空 (LSR 的第 6 位是 Transmit FIFO Empty)
        // bit 5 (THRE) 也可以，表示传输保持寄存器为空
        while unsafe { read_volatile(lsr) } & (1 << 5) == 0 {
            core::hint::spin_loop();
        }
 
        // 写入数据到发送寄存器
        unsafe {
            write_volatile(thr, c);
        }
    }
}
 
/// 实现 core::fmt::Write trait，让我们可以使用 write! 和 writeln! 宏
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // 将换行符 \n 转换为 \r\n，这是串口终端的通用做法
            if byte == b'\n' {
                self.putc(b'\r');
            }
            self.putc(byte);
        }
        Ok(())
    }
}
 
// ------ 全局静态实例和打印宏 ------
// 使用 lazy_static 或者更现代的 once_cell::sync::Lazy 可以避免 unsafe
// 但在早期内核，我们可以先用一个简单的 unsafe static mut
use core::sync::atomic::{AtomicUsize, Ordering};
 
// RK3566 UART2 基地址
// 请注意：参数UART2_BASE的数值来自
// Uboot的输出信息
// U-Boot 2017.09 (Jul 11 2025 - 07:42:38 +0000)

// Model: Rockchip RK3568 Evaluation Board
// MPIDR: 0x81000000
// PreSerial: 2, raw, 0xfe660000 <<- 这里
// DRAM:  2 GiB
// Sysmem: init
// Relocation Offset: 7d221000
// Relocation fdt: 7b9f8688 - 7b9fecd0
// CR: M/C/I
// Using default environment
const UART2_BASE: usize = 0xFE66_0000;
 
// 我们用一个全局的、可变的静态变量来代表我们的 UART 控制台
// 注意：这在单核环境下是安全的，但在多核启动前需要加锁
static mut GLOBAL_UART: AtomicUsize = AtomicUsize::new(0);
 
/// 初始化全局 UART
pub fn init_global_uart() {
    unsafe {
        GLOBAL_UART.store(UART2_BASE, Ordering::Relaxed);
    }
}
 
/// 获取全局 UART 的一个可变引用
fn global_uart() -> Uart {
    let base = unsafe { GLOBAL_UART.load(Ordering::Relaxed) };
    Uart::new(base)
}
 
// 定义我们自己的打印宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}
 
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
 
// 打印宏的后端实现
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut writer = global_uart();
    writer.write_fmt(args).unwrap();
}
 
 
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 初始化我们的全局 UART
    init_global_uart();
 
    // 现在可以使用 println! 宏了
    println!("--- Booting Rust OS on RK3566 ---");
    println!();
    println!("Hello from Rust!");
    
    let a = 10;
    let b = 20;
    println!("We can even print numbers: {} + {} = {}", a, b, a + b);
    
    println!();
    println!("System halt.");
 
    // 内核执行完毕，进入无限循环
    loop {}
}