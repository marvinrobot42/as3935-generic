# AS3935 &emsp; 
[![crates.io](https://img.shields.io/crates/v/bmp38x-ya)](https://crates.io/crates/as3935-generic)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/marvinrobot42/as3935a-generic)
[![Documentation](https://docs.rs/bmp38x-ya/badge.svg)](https://docs.rs/as3935-generic)

## A Rust crate for ScioSense/Franklin AS3935 lightning detector sensor for generic embedded use.

### Github:  <https://github.com/marvinrobot42/as3935-generic.git>

### AS3935 website: <https://www.sciosense.com/as3935-franklin-lightning-sensor-ic/>

The ScioSense/Franklin AS3935 lightning detector sensor with I2C interface.

All I2C API functions are implemented. 

### Features

- uses embedded-hal version 1.0.x
- no_std embedded compatible
- async support included as a feature, default is sync or blocking
- designed for embedded use (ESP32-C3, -C6 and -S3 and Raspberry Pi, any embedded-hal compatible)
- examples included (soon!)
- All IO functions implemented
- configurable threshold and sensivity and interrupt triggers
- set indoor or outdoor sensor location
 


  

#### Notes

Developed using Sparkfun AS3935 sensor model: https://www.sparkfun.com/sparkfun-lightning-detector-as3935.html



### Recent version history

  - 0.1.0  Initial release



## Usage
----

Add the dependency to `Cargo.toml`.

~~~~toml
[dependencies.as3935-generic]
version = "0.1"
~~~~
 


### Simple Example

A more complete example is in the repository examples path
~~~~rust

#![no_std]
#![no_main]

use esp_hal::i2c::master::{Config, I2c};
use esp_hal::delay::Delay;

use as3935_generic::{AS3935, constants::DeviceAddress::{self, AD1_0_AD0_1, AD1_1_AD0_0}, data::Location};

use log::info;
...


#[main]
fn main() -> ! {

  ...as per your hardware hal, below is ESP32-C6 no_std style

  esp_println::logger::init_logger_from_env();

  let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
  let peripherals = esp_hal::init(config);
  

  info!("creating i2c, delay and AS3935  instances");

  let mut i2c = I2c::new(peripherals.I2C0, Config::default()).unwrap()
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);


  let my_delay = Delay::new();
  let mut sensor = AS3935::new(i2c, DeviceAddress::default() as u8, my_delay);  
  sensor.set_indoor_outdoor(Location::Indoor).unwrap();
  let location = sensor.get_indoor_outdoor().unwrap();
  info!(" sensor location is {:?}", location);

  loop {
    info!("Hello ESP32-C6 no_std world...looping!");
   
    let int_reg = sensor.get_interrupt_register().unwrap();
    match int_reg {
      as3935_generic::data::INTType::NoiseHigh => log::info!(" AS3935 interrupt register = NoiseHigh"),
      as3935_generic::data::INTType::Disturber => log::info!(" AS3935 interrupt register = Disturber"),    as3935_generic::data::INTType::Lightning => log::info!(" AS3935 interrupt register = Lightning : {:?}", int_reg),
      as3935_generic::data::INTType::Nothing => log::info!(" AS3935 interrupt register = Nothing"),
    }
        
    let distance = sensor.get_distance_to_storm().unwrap();
    info!(" distance to storm front in km (last lightning strike) {:?}", distance);  // distance is also in INTType::Lightning above

    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(10000) {
      ;
    }
  }
    

}
~~~~

### For async set as3935-generic dependency features = ["async"] and AS3935::new method requires async I2C and delay 
###    parameters.  Default features is sync (blocking)


### License
----

You are free to copy, modify, and distribute this application with attribution under the terms of either

 * Apache License, Version 2.0
   ([LICENSE-Apache-2.0](./LICENSE-Apache-2.0) or <https://opensource.org/licenses/Apache-2.0>)
 * MIT license
   ([LICENSE-MIT](./LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

This project is not affiliated with nor endorsed in any way by Silicon Labs or Adafruit.
