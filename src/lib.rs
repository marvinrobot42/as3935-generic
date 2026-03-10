#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]


//#![feature(inherent_associated_types)]
pub mod error;

use crate::error::Error;

pub mod data;
use crate::data::{ AFE_GAIN, INTReg, INTType, LightningReg, Location, LocationMask, MinStrikes, Oscillator, StormFrontDistance, THRESHOLDS };

pub mod constants;

use crate::constants::DeviceAddress::{AD1_1_AD0_1, AD1_0_AD0_1, AD1_1_AD0_0, };
use crate::constants::{ AS3935_POWER_MASK, AS3935_REG0x00, AS3935_REG0x01, AS3935_REG0x02, AS3935_REG0x03, AS3935_REG0x3A, 
    AS3935_REG0x3B, AS3935_REG0x3C, AS3935_REG0x3D, AS3935_REG0x04, AS3935_REG0x05, AS3935_REG0x06, AS3935_REG0x07, 
    AS3935_REG0x08, AS3953_DIRECT_COMMAND };


#[cfg(not(feature = "async"))]
use embedded_hal::{i2c::I2c, delay::DelayNs};
#[cfg(feature = "async")]
use embedded_hal_async::{i2c::I2c as AsyncI2c, delay::DelayNs as AsyncDelayNs};

use log::{debug, info};
use libm::{pow, truncf};


/// the AS3935 device
pub struct AS3935<I2C, D> {
    /// I²C interface
    i2c: I2C,
    /// I²C device address
    address: u8,
    delayer: D,

}

#[cfg(not(feature = "async"))]
impl<I2C, D, E> AS3935<I2C, D>
where  
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    

    /// create new AS3935 driver with address and delay given
    /// the datasheet states the max speed is 400 kHz with just AS3935 on the bus and external 10 k pull ups
    /// otherwise use 4k7 pulls up and max speed is 100 kHz.  
    /// Best to use 100 kHz.
    /// use constant::DeviceAddress::default() with AD1 and AD0 pins high 
    pub fn new(i2c: I2C, address: u8, delayer: D) -> Self {
        log::debug!("new called");
        Self {
            i2c,
            address: address,
            delayer,
        }
    }


    /// give back the I2C interface
    pub fn release(self) -> I2C {
        self.i2c
    }

}

#[cfg(feature = "async")]
impl<I2C, D, E> AS3935<I2C, D>
where  
    I2C: AsyncI2c<Error = E>,
    D: AsyncDelayNs,
{
    /// create new AS3935 driver with address and delay given
    /// the datasheet states the max speed is 400 kHz with just AS3935 on the bus and external 10 k pull ups
    /// otherwise use 4k7 pulls up and max speed is 100 kHz.  
    /// Best to use 100 kHz.
    /// use constant::DeviceAddress::default() with AD1 and AD0 pins high  
    pub fn new(i2c: I2C, address: u8, delayer: D) -> Self {
        debug!("new called");
        Self {
            i2c,
            address: address,
            delayer,
        }
    }

    /// give back the I2C interface
    pub fn release(self) -> I2C {
        self.i2c.release;
    }

}

#[maybe_async_cfg::maybe(
    sync(
        cfg(not(feature = "async")),
        self = "AS3935",
        idents(AsyncI2c(sync = "I2c"), AsyncDelayNs(sync = "DelayNs"))
    ),
    async(feature = "async", keep_self)
)]

impl<I2C, D, E> AS3935<I2C, D>
where  
    I2C: AsyncI2c<Error = E>,
    D: AsyncDelayNs,
{

    // command_buf is an u8 array that starts with command byte followed by command data byte(s)
    async fn write_command<const N: usize>(&mut self, command_buf: [u8; N] ) -> Result<(), Error<E>> {
        // debug!("write_command : {:#?}", command_buf);
        self.i2c
            .write(self.address, &command_buf).await
            .map_err(Error::I2c)?;
        Ok(())
    }

    async fn read_register( &mut self, register_address: u8, buffer: &mut [u8] ) -> Result<(), Error<E>> {
        let mut command_buffer = [0u8; 1];
        command_buffer[0] = register_address;
        // let mut result_buffer = [0u8; N];
        self.i2c
            .write_read(self.address, &command_buffer, buffer).await
            .map_err(Error::I2c)?;
        Ok(())
    }

    

    /// calibrate both internal oscillators, first tune your antenna
    pub async fn calibrate_osc(&mut self) -> Result<(), Error<E>> {
        debug!("in calibrate_osc");
        self.write_command([AS3935_REG0x3D, AS3953_DIRECT_COMMAND]).await?;
        self.display_oscillator(true, Oscillator::TRCO);
        self.delayer.delay_ms(2).await;
        self.display_oscillator(false, Oscillator::TRCO);

        // check if osc started
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x3B, &mut result_buf).await?;
        let srco: bool = ((result_buf[0] & 0x40) >> 6) != 0;  // 1 = success, do not care about bit 5 
        self.read_register(AS3935_REG0x3A, &mut result_buf).await?;
        let trco: bool = ((result_buf[0] & 0x40) >> 6) != 0;  // 1 = success, do not care about bit 5
        if (srco || trco) {
            return Err(Error::OscFailedCalibration);
        } else {
            return Ok(());
        }
    }

        
    /// reset all settings to defaults
    pub async fn reset_settings(&mut self) -> Result<(), Error<E>> {
        debug!("in reset_settings");
        self.write_command([AS3935_REG0x3C, AS3953_DIRECT_COMMAND]).await?;
        Ok(())
    }

    /// read lightning energy (unit-less as per datasheet)
    pub async fn get_lightning_energy(&mut self) -> Result<u32, Error<E>> {
        debug!("in get_lightning_energy");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x06, &mut result_buf).await?;
        let mmsb: u8 = (result_buf[0] & 0x3f);
        self.read_register(AS3935_REG0x05, &mut result_buf).await?;
        let msb: u8 = result_buf[0];
        self.read_register(AS3935_REG0x04, &mut result_buf).await?;
        let lsb: u8 = result_buf[0];
        let bytes: [u8; 4] = [0x00, mmsb, msb, lsb];
        let value: u32 = u32::from_be_bytes(bytes);
        return Ok(value);

    }

    /// display oscillator on IRQ pin
    /// state = true to turn on, false to turn off
    /// use a logic analyzer or oscilloscope to see the clock signal.
    pub async fn display_oscillator(&mut self, state: bool, osc: Oscillator) -> Result<(), Error<E>> {
        debug!("in display_oscillator");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x08, &mut result_buf).await?;
        let mut value : u8 = result_buf[0] | (osc as u8); 
        if (state)  { //  turn on display
            self.write_command([AS3935_REG0x08,  value]).await?;
        } else { // turn off
            value = result_buf[0] & !(osc as u8);
            self.write_command([AS3935_REG0x08, value]).await?;
        }
        Ok(())
    }


    /// get distance to storm front in km.  > 40 km == OutOfRange
    pub async fn get_distance_to_storm(&mut self) -> Result<StormFrontDistance, Error<E>> {
        debug!("in get_distance_to_storm");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x07, &mut result_buf).await?;
        let distance: u8 = result_buf[0] & 0x3f;
        let mut storm_front_distance: StormFrontDistance = StormFrontDistance::OutOfRange;
        if (distance == 0x3f) {
            storm_front_distance = StormFrontDistance::OutOfRange;
        } else if (distance == 0x01) {
            storm_front_distance = StormFrontDistance::Overhead;
        } else {
            storm_front_distance = StormFrontDistance::Range_km(distance);
        }
        Ok(storm_front_distance)
    }

    /// power_down
    /// note:  the TRCO oscillator must then be recalibrated after power_down() call which is built into wakeup()
    pub async fn power_down(&mut self) -> Result<(), Error<E>> {
        debug!("in power_down");
        self.write_command([AS3935_REG0x00, AS3935_POWER_MASK ]).await?;
        Ok(())
    } 

    /// set indoor-outdoor location of sensor
    pub async fn set_indoor_outdoor(&mut self, location:  Location) -> Result<(), Error<E>> {
        debug!("in set_indoor_outdoor({:?})", location);
        let mut location_mask = LocationMask::Indoor;
        info!("  location is {:?} hex = {:02x}", location, location as u8);
    
        if (location == Location::Indoor) {
            location_mask = LocationMask::Indoor;
        } else {
            location_mask = LocationMask::Outdoor;
        }

        debug!("  location_mask = {:02x}", location_mask as u8);
        let mut afe_gain: AFE_GAIN = AFE_GAIN(0x00);
        afe_gain.set_location_mask(location_mask as u8);
        debug!("  afe_gain is {:02x}", afe_gain.0);
        self.write_command([AS3935_REG0x00, (afe_gain.0 as u8)]).await?;
        Ok(()) 
    }

    /// get indoor-outdoor location from sensor
    pub async fn get_indoor_outdoor(&mut self) -> Result<Location, Error<E>> {
        debug!("in get_indoor_outdoor");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x00, &mut result_buf).await?;
        debug!("  reg0x00 = {:02x}", result_buf[0]);
        let afe_gain: AFE_GAIN = AFE_GAIN(result_buf[0]);
        debug!("  afe_gain in hex = {:02x}", afe_gain.0);
        debug!("  afe_gain = {:?}", afe_gain);
        let mut where_is_it = Location::Indoor;
        if (afe_gain.get_location_mask() == LocationMask::Indoor) {
            where_is_it = Location::Indoor;
        } else if (afe_gain.get_location_mask() == LocationMask::Outdoor) {
            where_is_it = Location::Outdoor;
        } else {
            return Err(Error::ResponseError);
        }
        return Ok(where_is_it);
    }

    /// get interrupt register:  useful to see what the AS3935 detected
    pub async fn get_interrupt_register(&mut self) -> Result<INTType, Error<E>> {
        debug!("in get_interrupt_register");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x03, &mut result_buf).await?;
        let int_reg: INTReg = INTReg(result_buf[0]);
        let int_type: INTType = int_reg.get_int_type();
        Ok(int_type)
    }

    //  /// read_interrupt_register, first delay 2msec as per datasheet
    // pub async fn read_interrupt_register(&mut self) -> Result<INTType, Error<E>>  {
    //     debug!("in read_interrupt_register";)
    //     let mut result_buf: [u8; 1] = [0; 1];
    //     self.delayer.delay_ms(2).await;
    //     self.read_register(AS3935_REG0x03, &mut result_buf).await?;
    //     let int_reg: INTReg = INTReg(result_buf[0]);
    //     Ok(int_reg.get_int_type())
    // }

    /// wakup, and then calibrate the oscillators
    pub async fn wakeup(&mut self) -> Result<(), Error<E>> {
        debug!("in wakeup");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x3A, &mut result_buf).await?;
        let mut afe_gain: AFE_GAIN = AFE_GAIN(result_buf[0]);
        afe_gain.set_powerdown(false);  // false is == 0 to power up
        self.write_command([AS3935_REG0x3A, afe_gain.0]).await?;
        self.calibrate_osc().await?;
        Ok(())

    }

    /// set watchdog threshold, threshold < 11
    pub async fn set_watchdog_threshold(&mut self, threshold: u8 ) -> Result<(), Error<E>> {
        debug!("in set_watchdog_threshold");
        if (threshold > 10) {
            return Err(Error::ValueLimit);
        }
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x01, &mut result_buf).await?;
        let mut threshold_reg: THRESHOLDS = THRESHOLDS(result_buf[0]);
        threshold_reg.set_wd_threshold(threshold as u8);
        self.write_command([AS3935_REG0x01, threshold_reg.0]).await?;
        Ok(())
    }

    /// get watchdog threshold
    pub async fn get_watchdog_threshold(&mut self) -> Result<u8, Error<E>> {
        debug!("in get_watchdog_threshold");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x01, &mut result_buf).await?;
        let threshold_reg: THRESHOLDS = THRESHOLDS(result_buf[0]);
        let threshold = threshold_reg.get_wd_threshold();
        Ok(threshold)
    }

    /// set noise floor level, level < 8
    pub async fn set_noise_level(&mut self, level: u8 ) -> Result<(), Error<E>> {
        debug!("in set_noise_level");
        if (level > 7) {
            return Err(Error::ValueLimit);
        }
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x01, &mut result_buf).await?;
        let mut threshold_reg: THRESHOLDS = THRESHOLDS(result_buf[0]);
        threshold_reg.set_noise_floor(level);
        self.write_command([AS3935_REG0x01, threshold_reg.0]).await?;
        Ok(())
    }

    /// get noise floor level
    pub async fn get_noise_level(&mut self) -> Result<u8, Error<E>> {
        debug!("in get_noise_level");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x01, &mut result_buf).await?;
        let threshold_reg: THRESHOLDS = THRESHOLDS(result_buf[0]);
        let level = threshold_reg.get_noise_floor();
        Ok(level)
    }

    /// set lightning threshold (minimum number of lightning strikes)
    pub async fn set_lightning_threshold(&mut self, threshold: MinStrikes ) -> Result<(), Error<E>> {
        debug!("in set_lightning_threshold");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x02, &mut result_buf).await?;
        let mut lightning_reg: LightningReg = LightningReg(result_buf[0]);
        lightning_reg.set_min_strikes(threshold as u8);
        self.write_command([AS3935_REG0x02, lightning_reg.0]).await?;
        Ok(())
    }

    /// get lightning threshold (minimum number of lightning strikes)
    pub async fn get_lightning_threshold(&mut self) -> Result<MinStrikes, Error<E>> {
        debug!("in get_lightning_threshold");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x02, &mut result_buf).await?;
        let lightning_reg: LightningReg = LightningReg(result_buf[0]);
        let min_strikes: MinStrikes = lightning_reg.get_min_strikes();
        Ok(min_strikes)
    }

    /// clear statistics :  clears the number of lightning strikes detected in last 15 minutes
    pub async fn clear_statistics(&mut self) -> Result<(), Error<E>> {
        debug!("in clear_statistics");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x02, &mut result_buf).await?;
        let mut lightning_reg: LightningReg = LightningReg(result_buf[0]);
        lightning_reg.set_clear_stats(true);
        debug!("  writing lightning_reg = {:?}", lightning_reg);
        self.write_command([AS3935_REG0x02, lightning_reg.0]).await?;
        Ok(())
    }

    /// set mask disturber:  defines if "disturbers" trigger interrupts, default is false == not masked
    pub async fn set_mask_disturber(&mut self, enable: bool) -> Result<(), Error<E>> {
        debug!("in set_mask_disturber");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x03, &mut result_buf).await?;
        let mut int_reg: INTReg = INTReg(result_buf[0]);
        int_reg.set_mask_dist(enable);
        self.write_command([AS3935_REG0x03, int_reg.0]).await?;
        Ok(())
    }

    /// get mask disturber: whether disturbers triggers interrupts
    pub async fn get_mask_disturber(&mut self) -> Result<bool, Error<E>> {
        debug!("in get_mask_disturber");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x03, &mut result_buf).await?;
        let int_reg: INTReg = INTReg(result_buf[0]);
        Ok(int_reg.get_mask_dist())
    }

    /// set spike rejection sensitivity  < 16
    pub async fn set_spike_rejection(&mut self, sensitivity: u8) -> Result<(), Error<E>> {
        debug!("in set_spike_rejection");
        if (sensitivity > 15) {
            return Err(Error::ValueLimit);
        }
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x02, &mut result_buf).await?;
        let mut lightning_reg: LightningReg = LightningReg(result_buf[0]);
        lightning_reg.set_spike_reject(sensitivity);
        self.write_command([AS3935_REG0x02, lightning_reg.0]).await?;
        Ok(())
    }

    /// get spike rejection sensitivity
    pub async fn get_spike_rejection(&mut self) -> Result<u8, Error<E>> {
        debug!("in get_spike_rejection");
        let mut result_buf: [u8; 1] = [0; 1];
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x02, &mut result_buf).await?;
        let mut lightning_reg: LightningReg = LightningReg(result_buf[0]);
        let sensitivity = lightning_reg.get_spike_reject();
        Ok(sensitivity)
    }

    /// set antenna frequency division ratio for antenna tuning
    /// ratios are:  16, 32, 64 or 128
    pub async fn set_antenna_div_ratio(&mut self, ratio: u8) -> Result<(), Error<E>> {
        debug!("in set_antenna_div_ratio ( {} )", ratio);
        let mut value: u8 = 0;
        if (ratio == 16) {
            value = 0;
        } else if (ratio == 32) {
            value = 1;
        } else if { ratio == 64} {
            value = 2;
        } else if (ratio == 128) {
            value = 3;
        } else {
            return Err(Error::ValueLimit);
        }

        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x03, &mut result_buf).await?;
        let mut int_reg: INTReg = INTReg(result_buf[0]);
        int_reg.set_freq_div(value);
        //info!("  writing value {:02x} and int_reg = {:?}", value, int_reg);
        self.write_command([AS3935_REG0x03, int_reg.0]).await?;
        Ok(())
    }

    /// get antenna frequency division ratio for antenna tuning
    pub async fn get_antenna_div_ratio(&mut self) -> Result<u8, Error<E>> {
        debug!("in get_antenna_div_ratio");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x03, &mut result_buf).await?;
        let mut int_reg: INTReg = INTReg(result_buf[0]);
        let value = int_reg.get_freq_div();
        let return_value: u8 = (1 << (value + 4));
        Ok((return_value))
    }

    /// set tuning cap for antenna
    /// farads must be <= 128 and modulo 8,  specifically from 0 to 120 pF in steps of 8 pF (modulo 8) 
    pub async fn set_tuning_cap(&mut self, p_farads: u8) -> Result<(), Error<E>> {
        debug!("in set_tuning_cap( {:02x} )", p_farads);
        if (p_farads > 128) {
            return Err(Error::ValueLimit);
        } else if ((p_farads % 8) != 0) {
            debug!("  bummer, not modulo 8");
            return Err(Error::ValueLimit);
        }
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x08, &mut result_buf).await?;
        let mut new_value: u8 = (result_buf[0]) | ((p_farads >> 3) & 0x17);
        //info!("  new tuning cap value to write is {:02x}", new_value);
        self.write_command([AS3935_REG0x08, new_value]).await?;

        Ok(())
    }

    /// get tuning cap (internal) for antenna in pF, manufacturer default is 0 pF if not changed
    pub async fn get_tuning_cap(&mut self) -> Result<u8, Error<E>> {
        debug!("in get_tuning_cap");
        let mut result_buf: [u8; 1] = [0; 1];
        self.read_register(AS3935_REG0x08, &mut result_buf).await?;
        let p_farads: u8 = (result_buf[0] & 0x1f) << 3;  // bit [3:0] in steps of 8 pF 
        Ok(p_farads)
    }


}
