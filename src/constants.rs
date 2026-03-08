// AS3935 registers

#![allow(nonstandard_style)]
pub const AS3935_REG0x00: u8 = 0x00;  /// AFE Gain 
pub const AS3935_REG0x01: u8 = 0x01;  /// Int pin threshold 
pub const AS3935_REG0x02: u8 = 0x02;  /// Lightning sensivity
pub const AS3935_REG0x03: u8 = 0x03;  /// Interrupt stuff
pub const AS3935_REG0x04: u8 = 0x04;  /// Lightning energy LSB
pub const AS3935_REG0x05: u8 = 0x05;  /// Lightning energy MSB
pub const AS3935_REG0x06: u8 = 0x06;  /// Lightning eneergy "Mostest" Sigificant Byte
pub const AS3935_REG0x07: u8 = 0x07;  /// Distance to front of storm
pub const AS3935_REG0x08: u8 = 0x08;  /// Osc Frequency to INT (IRQ) pin
pub const AS3935_REG0x3A: u8 = 0x3a;  /// Calibrate TRC0 oscillator
pub const AS3935_REG0x3B: u8 = 0x3b;  /// Calibrate SRC0 oscillator
pub const AS3935_REG0x3C: u8 = 0x3c;  /// Reset All setting to default values
pub const AS3935_REG0x3D: u8 = 0x3d;  /// Calibrate internal RC oscillator automatically

pub const AS3953_DIRECT_COMMAND: u8 = 0x96;
pub const AS3935_POWER_MASK: u8 = 0xfe;

#[repr(u8)]
/// AS3935 I2C device address
#[derive(Debug, Clone, Copy, Default)]
pub enum DeviceAddress {
    /// it has two address pins called AD0 amd AD1
    #[default]
    AD1_1_AD0_1 = 0x03, /// AD0 and AD1 are high, the default
    AD1_1_AD0_0 = 0x02, ///  AD1 high, AD0 low
    AD1_0_AD0_1 = 0x01, // AD1 low, AD0 high
}

impl From<DeviceAddress> for u8 {
    fn from(value: DeviceAddress) -> Self {
        match value {
            DeviceAddress::AD1_1_AD0_1 => 0x03, // AD0 and AD1 are high
            DeviceAddress::AD1_1_AD0_0 => 0x02, // AD1 high, AD0 low
            DeviceAddress::AD1_0_AD0_1 => 0x01, // AD1 low, AD0 high

        }
    }
}

