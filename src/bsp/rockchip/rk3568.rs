// src/bsp/rockchip/rk3568.rs

//! RK3568芯片的内存映射地址常量
//!
//! 信息来源: "Rockchip RK3568 TRM Part1 V1.1-20210301.pdf"

// TRM v1.1, Part 1, Page 12, "Table 1-1Address Mapping"
// TSADC (Temperature-Sensor ADC) 模块的物理基地址
pub const TSADC_PHYS_BASE: usize = 0xFE710000;

// 从U-Boot启动日志 "PreSerial: 2, raw, 0xfe660000" 可知调试串口为UART2.
// TRM v1.1, Part 1, Page 12, "Table 1-1Address Mapping" 确认了UART2的地址.
pub const UART2_PHYS_BASE: usize = 0xFE660000;
