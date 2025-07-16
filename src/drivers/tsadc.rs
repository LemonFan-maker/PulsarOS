// src/drivers/tsadc.rs

//! Rockchip TSADC (温度传感器)驱动.

use core::ptr::{read_volatile, write_volatile};

// 寄存器偏移量 (来源: TRM v1.1, Part 1, p.667)
const TSADC_USER_CON: usize = 0x00;
const TSADC_INT_PD:   usize = 0x0C;
const TSADC_DATA0:    usize = 0x20;

// `TSADC_USER_CON` 寄存器的位定义 (来源: TRM v1.1, Part 1, p.668)
const START_OF_CONVERSION: u32 = 1 << 0;
const ADC_POWER_CTRL:      u32 = 1 << 3;

/// 代表一个TSADC硬件实例.
pub struct Tsadc {
    base_addr: usize,
}

impl Tsadc {
    /// 创建一个新的TSADC实例.
    ///
    /// # Safety
    ///
    /// `base_addr` 必须是有效的TSADC控制器物理基地址.
    /// 代码假设使用恒等映射（Identity-mapped）的虚拟地址.
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    /// 读取CPU核心的温度.
    ///
    /// 返回值为摄氏度.
    pub fn read_temperature(&self) -> f32 {
        let user_con_ptr = (self.base_addr + TSADC_USER_CON) as *mut u32;
        let int_pd_ptr   = (self.base_addr + TSADC_INT_PD) as *mut u32;
        let data0_ptr    = (self.base_addr + TSADC_DATA0) as *const u32;

        unsafe {
            // 1. 上电并选择通道0 (CPU温度传感器)
            write_volatile(user_con_ptr, ADC_POWER_CTRL);

            // 2. 开始一次转换
            write_volatile(user_con_ptr, ADC_POWER_CTRL | START_OF_CONVERSION);

            // 3. 轮询中断状态位，等待转换完成
            while (read_volatile(int_pd_ptr) & 1) == 0 {
                unsafe {
                    core::arch::asm!("nop");
                }
            }

            // 4. 读取原始数据 (低12位有效)
            let raw_data = read_volatile(data0_ptr) & 0xFFF;

            // 5. 清除中断挂起状态 (写1清除)
            write_volatile(int_pd_ptr, 1);
            
            // 6. 关闭ADC电源以省电
            write_volatile(user_con_ptr, 0);

            // 7. 根据TRM和Linux内核驱动中的公式将原始值转换为摄氏度
            //    公式: T = (ADC_code - 1859) / -7.53
            (raw_data as f32 - 1859.0) / -7.53
        }
    }
}
