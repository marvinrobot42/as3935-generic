#![no_std]
#![no_main]

/***********************************************************************************
 * Recommend using the ESP32-C3 or -C6 over the Pico-2.  The pico-2 is not developer
 * friendly for Rust and the pico-2-W is even worse for Wifi usage.  A Pi Debug
 * Probe is required to see log statements.  Also, the Debug Probe uses all CPUs/
 * core available when active turning your laptop into a toaster.  But if you are up
 * for a challenge...
 * 
 * This project uses Pi Debug Probe with defmt for logging.
 **********************************************************************************/

use as3935_generic::data::StormFrontDistance;
use embassy_rp as hal;
use embassy_executor::Spawner;
use embassy_rp::block::ImageDef;
use embassy_time::Timer;

use embassy_time::Delay;


//Panic Handler
use {panic_probe as _};

// Interrupt Binding
use embassy_rp::peripherals::I2C0;
use embassy_rp::{bind_interrupts, i2c};

// I2C
use embassy_rp::i2c::{Config as I2cConfig, I2c};

use as3935_generic::{AS3935, constants::DeviceAddress::{self, AD1_0_AD0_1, AD1_1_AD0_0}, data::Location};

use defmt::{info, warn, error};
use defmt_rtt as _; // global logger


/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});




#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let p = embassy_rp::init(Default::default());

    let sda = p.PIN_16;
    let scl = p.PIN_17;

    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 100_000; //400kHz

    let i2c_bus = I2c::new_async(p.I2C0, scl, sda, Irqs, i2c_config);

    let my_delay = Delay;


    let mut sensor = AS3935::new(i2c_bus, DeviceAddress::default() as u8, my_delay);  
    sensor.set_indoor_outdoor(Location::Indoor).await.unwrap();
    let _location = sensor.get_indoor_outdoor().await.unwrap();
 

    loop{

        let int_reg = sensor.get_interrupt_register().await.unwrap();
        match int_reg {
            as3935_generic::data::INTType::NoiseHigh => info!(" AS3935 interrupt register = NoiseHigh"),
            as3935_generic::data::INTType::Disturber => info!(" AS3935 interrupt register = Disturber"),
            as3935_generic::data::INTType::Lightning => {
                info!(" AS3935 interrupt register = Lightning");
                let distance = sensor.get_distance_to_storm().await.unwrap();
                match distance {
                    as3935_generic::data::StormFrontDistance::OutOfRange => info!(" distance = OutOfRange"),
                    as3935_generic::data::StormFrontDistance::Range_km(_) => {
                        if let StormFrontDistance::Range_km(val) = distance {
                            info!(" distance = {} km", val);
                        }
                    }
                    as3935_generic::data::StormFrontDistance::Overhead => info!(" distance = Overhead"),
                }

                //info!(" distance to storm front is (km) {:?}",sensor.get_distance_to_storm().await.unwrap() );  // in line above also
                info!(" lightning strike energy is {}", sensor.get_lightning_energy().await.unwrap());
            },
            as3935_generic::data::INTType::Nothing => info!(" AS3935 interrupt register = Nothing"),
        }
        Timer::after_secs(5).await;
    }   
}




// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"test-one"),
    embassy_rp::binary_info::rp_program_description!(
        c"your program description"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];



// End of file

