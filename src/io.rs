//! IO helper functions.

use i2cdev::core::I2CDevice;

/// Read a contiguous series of `buf.length` registers from the given
/// I2C device `bus`, starting with `reg`.
pub fn read_reg<I>(bus: &mut I, reg: u8, buf: &mut [u8]) -> Result<(), I::Error>
    where I: I2CDevice,
{
    try!(bus.write(&[reg]));
    bus.read(buf)
}
