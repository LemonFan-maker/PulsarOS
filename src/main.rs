// main.rs

#![no_std]
#![no_main]

use core::panic::PanicInfo;

// 模块声明
mod bsp;
mod drivers;

// 从其他文件导入
use bsp::rockchip::rk3568;
use drivers::tsadc::Tsadc;

/// 内核入口点
///
/// `_start` 函数 (在 exceptions.s 中) 会跳转到这里.
#[no_mangle]
pub extern "C" fn kernel_main() {
    // 初始化串口，以便我们可以打印信息
    // 注意: 我们假设U-Boot已经完成了必要的时钟和引脚配置
    // 并且内存是恒等映射的 (虚拟地址 == 物理地址)
    unsafe {
        drivers::uart::init(rk3568::UART2_PHYS_BASE);
    }

    println!("\n[+] Rust OS Kernel for RK3568");
    println!("[+] Kernel entry point `kernel_main` reached.");

    // 创建TSADC驱动实例
    let tsadc = unsafe { Tsadc::new(rk3568::TSADC_PHYS_BASE) };
    
    println!("[+] Reading CPU temperature...");

    // 读取温度
    let temp = tsadc.read_temperature();

    // 打印结果
    // 注意: 浮点数打印在 no_std 环境下可能需要额外配置，
    // 但我们的打印宏很简单，这里可以工作。
    println!("[+] CPU Temperature: {} C", temp as i32);
    
    println!("[+] All tasks finished. Halting.");
    // 内核无事可做，进入死循环
    loop {}
}

/// Panic处理函数
///
/// 在 `no_std` 环境下，当panic发生时，此函数会被调用。
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n*** KERNEL PANIC ***");
    println!("Panic info: {}", info);
    loop {}
}