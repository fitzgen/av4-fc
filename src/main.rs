#![deny(missing_docs)]
#![deny(warnings)]

//! This program reads measurements from an MPU-9150 inertial
//! measurement unit attached via I2C.

extern crate byteorder;
extern crate i2cdev;

pub mod fc;
pub mod io;

use i2cdev::linux::LinuxI2CDevice;
use std::env;
use std::time::Duration;
use std::thread::sleep;

fn main() {
    let dev = env::args()
        .nth(1)
        .expect(&format!("Usage: {} /dev/i2c-?",
                         env::args().nth(0).unwrap_or("program".into())));

    let bus = LinuxI2CDevice::new(&dev, 0x68).expect(&format!("opening {} failed", &dev));
    let mut flight_controller = fc::FlightController::new(bus).unwrap();

    let delay = Duration::from_millis(200);
    while let Ok(sample) = {
        sleep(delay);
        flight_controller.read_sample()
    } {
        println!("{:?}", sample);
    }
}
