// ESP32-C6 based no_std example

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::clock::{CpuClock, ModemClockController};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use log::info;

use esp_hal::i2c::master::{Config, I2c};
use esp_hal::delay::Delay;

use as3935_generic::{AS3935, constants::DeviceAddress::{self, AD1_0_AD0_1, AD1_1_AD0_0}, data::Location};

#[main]
fn main() -> ! {

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
 

    info!("creating i2c, delay and AS3935  instances");

    let mut i2c = I2c::new(peripherals.I2C0, Config::default()).unwrap()
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7);


    let my_delay = Delay::new();

    info!("creating AS3935 device");
    let mut sensor = AS3935::new(i2c, DeviceAddress::default() as u8, my_delay);
    info!("created AS3935 device");
    sensor.set_indoor_outdoor(Location::Indoor).unwrap();
    let location = sensor.get_indoor_outdoor().unwrap();
    info!(" sensor location is {:?}", location);

    loop {
        info!("Hello ESP32-C6 no_std world...looping!");
   
        let int_reg = sensor.get_interrupt_register().unwrap();
        match int_reg {
            as3935_generic::data::INTType::NoiseHigh => log::info!(" AS3935 interrupt register = NoiseHigh"),
            as3935_generic::data::INTType::Disturber => log::info!(" AS3935 interrupt register = Disturber"),
            as3935_generic::data::INTType::Lightning => {
                log::info!(" AS3935 interrupt register = Lightning : {:?}", int_reg);
                info!(" distance to storm front is (km) {:?}",sensor.get_distance_to_storm().unwrap() );
                info!(" lightning strike energy is {}", sensor.get_lightning_energy().unwrap());
            },
            as3935_generic::data::INTType::Nothing => log::info!(" AS3935 interrupt register = Nothing"),
        }
        

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(10000) {
        ;
    }
  }
    

}



fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, bmp38x-ya world!");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let sda = pins.gpio6; // esp32-c3  has pins.gpio0 , check your board schematic
    let scl = pins.gpio7; // esp32-c3  haspins.gpio1, check your board schematic
    let i2c = peripherals.i2c0;
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c_bus = I2cDriver::new(i2c, sda, scl, &config)?;

    
    let mut delay: Delay = Default::default();
    let mut sensor = BMP38x::new(i2c_dev,DeviceAddress::Secondary as u8, &mut delay);
    log::info!("created BMP38x device, calling init_device next");
    sensor.init_device().unwrap();
    log::info!("BMP38x init_device done");

    let new_config = const {
        Bmp3Configuration::builder()
            .power_mode(PowerMode::Normal)  // very important!
            .temperature_enable(true)
            .pressure_enable(true)
            .over_sampling_temp(Over_Sampling::ULTRA_LOW_POWER) // ultra_low ok
            .over_sampling_press(Over_Sampling::HIGH_RESOLUTION)// standard ok
            .iir_filter_coef(FilterCoef::COEF_15) // CORF_15 ok
            .output_data_rate(Odr::ODR_12P5)  // ODR_50 ok,  ODR_12P5 for slower rate test
            .build()
    };

    // if you only need ULTRA_LOW_POWER or less resolution you can just set the power mode to normal with
    //  sensor.set_power_control(PowerControl::normal()).unwrap() and skip the set_bmp3_configuration call

    sensor.set_bmp3_configuration(new_config).unwrap();
    let config = sensor.get_bmp3_configuration().unwrap();  // read it back to see that it worked
    info!("new Bmp3Configuration = {:?}", config);
    FreeRtos::delay_ms(1000);



    loop {
        let status = sensor.get_status().unwrap();
        info!(" bmp38x status is  {:?}", status);
        if (status.get_temp_ready() && status.get_press_ready()) {  // without checking this you could get old data
            let sensor_measurement = sensor.read_measurements_with_altitude(383.5).unwrap();  // current elevation is 383.5 m
            // or let sensor_measurement = sensor.read_measurements().unwrap();  if sealevel pressure not used
            let intstatus = sensor.get_interrupt_status().unwrap();
            info!("  IntStatus is {:?}", intstatus);
            info!(" sensor_measurement = {:?}", sensor_measurement);
        } else {
            info!("  bmp38x data not ready");
        }
    }

    Ok(())

}
