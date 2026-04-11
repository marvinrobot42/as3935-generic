use anyhow::Result;
use linux_embedded_hal::{Delay, I2cdev};

use as3935_generic::{AS3935, constants::DeviceAddress::{self, AD1_0_AD0_1, AD1_1_AD0_0}, data::Location};

use std::thread;
use std::time::Duration;

use env_logger::Builder;
use log::{LevelFilter, info};
use std::io::Write;

fn main() -> Result<()> {

    let mut builder = Builder::from_default_env();

    builder
        .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();


    info!("Hello, linux Rust and AS3935 world!");

    let dev_i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let my_delay = Delay {};

    let mut sensor = AS3935::new(dev_i2c, DeviceAddress::default() as u8, my_delay);  
    sensor.set_indoor_outdoor(Location::Indoor).unwrap();
    let location = sensor.get_indoor_outdoor().unwrap();
    info!(" sensor location is {:?}", location);
    
    thread::sleep(Duration::from_secs(2));



    loop {
        info!("Hello RPi AS3935-generic world...looping!");
   
        let int_reg = sensor.get_interrupt_register().unwrap();
        match int_reg {
            as3935_generic::data::INTType::NoiseHigh => log::info!(" AS3935 interrupt register = NoiseHigh"),
            as3935_generic::data::INTType::Disturber => log::info!(" AS3935 interrupt register = Disturber"),
            as3935_generic::data::INTType::Lightning => {
                info!(" AS3935 interrupt register = Lightning : {:?}", int_reg);
                info!(" distance to storm front is (km) {:?}",sensor.get_distance_to_storm().unwrap() );  // in line above also
                info!(" lightning strike energy is {}", sensor.get_lightning_energy().unwrap());
            },
            as3935_generic::data::INTType::Nothing => log::info!(" AS3935 interrupt register = Nothing"),
        }

        thread::sleep(Duration::from_secs(10));
    
    }

}