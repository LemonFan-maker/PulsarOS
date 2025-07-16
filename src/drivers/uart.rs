// src/drivers/uart.rs

//! 一个极简的、基于轮询的串口驱动.

use core::fmt::{self, Write};

// UART寄存器偏移量
const UART_THR: usize = 0x00; // Transmitter Holding Register
const UART_LSR: usize = 0x14; // Line Status Register

// LSR位定义
const LSR_THRE: u8 = 1 << 5; // Transmitter Holding Register Empty

struct Uart {
    base_address: usize,
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // 等待发送FIFO为空
            while unsafe { core::ptr::read_volatile((self.base_address + UART_LSR) as *const u8) } & LSR_THRE == 0 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }
            // 发送一个字节
            unsafe {
                core::ptr::write_volatile((self.base_address + UART_THR) as *mut u8, byte);
            }
        }
        Ok(())
    }
}

// 全局唯一的UART writer实例
static mut UART_WRITER: Uart = Uart { base_address: 0 };

/// 初始化串口驱动
///
/// # Safety
///
/// 调用者必须确保传入的base_address是有效的UART基地址，
/// 并且此函数在内核中只被调用一次。
pub unsafe fn init(base_address: usize) {
    UART_WRITER.base_address = base_address;
}

// Rust的格式化宏需要这些辅助函数和宏定义

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    unsafe {
        UART_WRITER.write_fmt(args).unwrap();
    }
}

/// 打印到串口
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::uart::_print(format_args!($($arg)*)));
}

/// 打印到串口并换行
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

