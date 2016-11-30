//! The flight controller.

use byteorder::{BigEndian, ReadBytesExt};
use i2cdev::core::I2CDevice;
use io;
use std::error::Error;
use std::io as stdio;

/// Structure to hold measurements in real units.
#[derive(Debug)]
pub struct MPUSample {
    /// Acceleration X/Y/Z in g's
    pub accel: [f32; 3],
    /// Temperature in degrees Celsius
    pub temp: f32,
    /// Rotational velocity X/Y/Z in degrees/second
    pub gyro: [f32; 3],
}

/// The flight controller.
///
/// Samples the sensors, decides what course of action to take.
pub struct FlightController<I> {
    bus: I,
}

impl<I> FlightController<I>
    where I: I2CDevice,
          I::Error: Error + From<stdio::Error>
{
    /// Set up an MPU-9150's configuration registers.
    pub fn new(mut bus: I) -> Result<FlightController<I>, I::Error> {
        // This sensor has a "WhoAmI" register that, when read, should
        // always return 0x68. If we read that register and get a
        // different value, then this isn't an MPU-family IMU and we
        // shouldn't try to poke at it further.
        let mut buf = [0u8; 1];
        try!(io::read_reg(&mut bus, 0x75, &mut buf));
        if buf[0] != 0x68 {
            return Err(stdio::Error::new(stdio::ErrorKind::NotFound,
                                         "MPU-9150 WhoAmI returned wrong value")
                .into());
        }

        // Wake device up, using internal oscillator.
        try!(bus.write(&[0x6b, 0x00]));

        // Set configuration:
        // - Sample rate divider: 1kHz / 200
        // - Config: no FSYNC, low-pass filter at 5Hz
        // - Gyro config: full scale range at +/- 250 dps
        // - Accel config: full scale range at +/- 2g
        try!(bus.write(&[0x19, 199, 0x06, 0x00, 0x00]));

        Ok(FlightController { bus: bus })
    }

    /// Read an `MPUSample` from the given I2C device, which must have been
    /// initialized first using `setup`.
    pub fn read_sample(&mut self) -> Result<MPUSample, I::Error> {
        // This sensor family places the measured values in a contiguous
        // block of registers, which allows us to do a bulk read of all
        // of them at once. And it's important to do the read in bulk,
        // because this hardware locks the register values while we're
        // reading them so that none of the sampled values change
        // mid-read. If we read them byte-at-a-time, we could get a
        // high-order byte from an old sample and a low-order byte from
        // a new sample, and wind up with nonsense numbers.
        let mut buf = [0u8; (3 + 1 + 3) * 2];
        try!(io::read_reg(&mut self.bus, 0x3b, &mut buf));

        // If read_i16 returns an error, it will be of type stdio::Error.
        // However, we're supposed to return errors of the type
        // associated with the I2CDevice implementation we're using. So
        // above we constrained type E to have an implementation of the
        // From trait, which the try! macro will use to convert
        // stdio::Error to E as needed.
        let mut rdr = stdio::Cursor::new(buf);
        Ok(MPUSample {
            accel: [(try!(rdr.read_i16::<BigEndian>()) as f32) / 16384.0,
                    (try!(rdr.read_i16::<BigEndian>()) as f32) / 16384.0,
                    (try!(rdr.read_i16::<BigEndian>()) as f32) / 16384.0],
            temp: (try!(rdr.read_i16::<BigEndian>()) as f32) / 340.0 + 35.0,
            gyro: [(try!(rdr.read_i16::<BigEndian>()) as f32) / 131.0,
                   (try!(rdr.read_i16::<BigEndian>()) as f32) / 131.0,
                   (try!(rdr.read_i16::<BigEndian>()) as f32) / 131.0],
        })
    }
}
