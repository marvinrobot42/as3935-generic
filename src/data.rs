use core::default;

// no_std support
#[allow(unused_imports)]
#[allow(dead_code)]
// use libm::{exp, round, trunc};
use log::debug;
use bitfield::bitfield;
// use const_builder::ConstBuilder;
use log::info;


#[allow(unused_imports)] // for no_std use


/// A measurement result from the sensor.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Measurements {
    /// distance to storm front in km
    pub distanceToStorm: f64,
    /// energy of lightning strike in no specific units (see datasheet)
    pub energyStrike: f64,
}

/// location of sensor 
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum Location {
    #[default]
    Indoor, 
    Outdoor,  
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum LocationMask {
    #[default]
    Indoor =   0x09,   // was 0x12 shift right 1, bitfield shifts it
    Outdoor =  0x07,   // was 0x0e shift right 1, bitfield shifts it
}

impl From<u8> for LocationMask {
    fn from(v: u8) -> Self {
        match (v) {
            0x09 => Self::Indoor,
            0x07 => Self::Outdoor,
            _ => unreachable!(),
        }
    }
}


bitfield! {
    /// AS3935 Reg 0x00 AFE_GAIN register
    pub struct AFE_GAIN(u8);
    impl Debug;

    pub  bool, get_powerdown, set_powerdown: 0;  // force power down state with = 1
    pub  into LocationMask, get_location_mask, set_location_mask: 5,1;   // AFE Gain boost for indoor/outdoor lcoation

}

bitfield! {
    /// AS3935 Reg 0x01 Thresholds
    pub struct THRESHOLDS(u8);
    impl Debug;

    pub u8, get_noise_floor, set_noise_floor: 6, 4; // Noise Floor level
    pub u8, get_wd_threshold, set_wd_threshold: 3, 0;  // Watchdog threshold

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
#[allow(non_camel_case_types)]
#[repr(u8)]
/// Minimum Lightning Strikes to trigger interrupt
pub enum MinStrikes {
    #[default]
    ONE =   0x00,  /// 1
    FIVE =   0x01,  /// 5
    NINE  =   0x02,  /// 9
    SIXTEEN  =   0x03,  // 16
}

impl From<u8> for MinStrikes {
    fn from(v: u8) -> Self {
        match (v) {
            0x00 => Self::ONE,
            0x01 => Self::FIVE,
            0x02 => Self::NINE,
            0x03 => Self::SIXTEEN,
            _ => Self::ONE,
        }
    }
}

bitfield! {
    /// AS3935 Lightning register settings
    pub struct LightningReg(u8);
    impl Debug;

    pub bool, _, set_clear_stats: 6;    // Clear statistics
    pub into MinStrikes, get_min_strikes, set_min_strikes: 5, 4; // minimum number of lightning strike, default is 0b00        
    pub u8, get_spike_reject, set_spike_reject: 3, 0;   // spike rejection , default 0b0010

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
#[allow(non_camel_case_types)]
#[repr(u8)]
/// Interrupt Type : what triggered interrupt
pub enum INTType {
    #[default]
    NoiseHigh = 0b0001,
    Disturber = 0b0100,
    Lightning = 0b1000,
    Nothing   = 0b0000,
}

impl From<u8> for INTType {
    fn from(v: u8) -> Self {
        match (v) {
            0b0001 => Self::NoiseHigh,
            0b0100 => Self::Disturber,
            0b1000 => Self::Lightning,
            0b0000 => Self::Nothing,
            _ => unreachable!(),
        }
    }
}

bitfield! {
    /// AS3935 INT registry bits
    pub struct INTReg(u8);
    impl Debug;

    pub u8, get_freq_div, set_freq_div: 7, 6;   // Frequncy division ration for antenna tuning
    pub bool, get_mask_dist, set_mask_dist: 5;  // mask disturber
    pub into INTType, get_int_type, _: 3, 0;    // interrupt type
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Storm Front Distance
pub enum StormFrontDistance {
    /// storm front is out of range
    OutOfRange,
    /// storm front is in 5-40 km range
    Range_km(u8),
    /// storm front is overhead
    Overhead,
}

// lightning Strike Enum  ...phase this out
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum StrikeEnum {
//     Lightning(StormFrontDistance),
//     ExcessiveNoise,
//     Disturber,
// }

/// which internal oscillator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Oscillator {
    TRCO = 0b00100000,  //  System RCO at 32.768kHz
    SRCO = 0b01000000,  //  Timer RCO Oscillators 1.1MHz
    LCO  = 0b10000000,  //  Frequency of the Antenna
}

