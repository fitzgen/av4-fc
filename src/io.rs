//! IO helper functions.

use i2cdev::core::I2CDevice;

/// Read a contiguous series of `buf.length` registers from the given
/// I2C device `bus`, starting with `reg`.
pub fn read_reg<I>(bus: &mut I, reg: u8, buf: &mut [u8]) -> Result<(), I::Error>
    where I: I2CDevice
{
    try!(bus.write(&[reg]));
    bus.read(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use i2cdev::core::I2CDevice;
    use std::error;
    use std::fmt;

    #[derive(Clone, Copy, Debug)]
    struct MockError;

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(f, "MockError")
        }
    }

    impl error::Error for MockError {
        fn description(&self) -> &str {
            "MockError"
        }
    }


    #[derive(Debug)]
    enum MockOperation<'a> {
        Read(&'a [u8], Result<(), MockError>),
        Write(&'a [u8], Result<(), MockError>),
    }

    #[derive(Debug)]
    struct MockI2CDevice<I> {
        operations: I,
    }

    impl<'a, 'b, I> MockI2CDevice<I>
        where 'a: 'b,
              I: Iterator<Item = &'b MockOperation<'a>>
    {
        fn new<T>(iter: T) -> MockI2CDevice<I>
            where T: IntoIterator<IntoIter = I, Item = &'b MockOperation<'a>>
        {
            MockI2CDevice { operations: iter.into_iter() }
        }
    }

    impl<'a, 'b, I> I2CDevice for MockI2CDevice<I>
        where 'a: 'b,
              I: Iterator<Item = &'b MockOperation<'a>>
    {
        type Error = MockError;

        fn read(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
            match self.operations.next() {
                Some(&MockOperation::Read(mock_data, result)) => {
                    assert_eq!(data.len(), mock_data.len());
                    data.copy_from_slice(mock_data);
                    result
                }
                otherwise => {
                    panic!("Expected {:?}, found read with buf len {}",
                           otherwise,
                           data.len())
                }
            }
        }

        fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            match self.operations.next() {
                Some(&MockOperation::Write(expected_data, result)) => {
                    assert_eq!(data, expected_data);
                    result
                }
                otherwise => panic!("Expected {:?}, found write of {:?}", otherwise, data),
            }
        }

        fn smbus_write_quick(&mut self, _bit: bool) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn smbus_read_block_data(&mut self, _register: u8) -> Result<Vec<u8>, Self::Error> {
            unimplemented!()
        }

        fn smbus_read_i2c_block_data(&mut self,
                                     _register: u8,
                                     _len: u8)
                                     -> Result<Vec<u8>, Self::Error> {
            unimplemented!()
        }

        fn smbus_write_block_data(&mut self,
                                  _register: u8,
                                  _values: &[u8])
                                  -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn smbus_process_block(&mut self, _register: u8, _values: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn read_reg_ok() {
        let register = 0;

        let expected_write = [register];
        let expected_read = [10, 20, 30, 40, 50];

        let operations = [MockOperation::Write(&expected_write, Ok(())),
                          MockOperation::Read(&expected_read[..], Ok(()))];

        let mut dev = MockI2CDevice::new(&operations);

        let mut buf = [0, 0, 0, 0, 0];
        let result = read_reg(&mut dev, register, &mut buf[..]);

        assert!(result.is_ok());
        assert_eq!(buf, expected_read);
    }

    #[test]
    fn read_reg_write_fails() {
        let register = 0;

        let expected_write = [register];
        let operations = [MockOperation::Write(&expected_write, Err(MockError))];
        let mut dev = MockI2CDevice::new(&operations);

        let mut buf = [0, 0, 0, 0, 0];
        let result = read_reg(&mut dev, register, &mut buf[..]);

        assert!(result.is_err());
    }

    #[test]
    fn read_reg_read_fails() {
        let register = 0;

        let expected_write = [register];
        let expected_read = [10, 20, 30, 40, 50];

        let operations = [MockOperation::Write(&expected_write, Ok(())),
                          MockOperation::Read(&expected_read, Err(MockError))];
        let mut dev = MockI2CDevice::new(&operations);

        let mut buf = [0, 0, 0, 0, 0];
        let result = read_reg(&mut dev, register, &mut buf[..]);

        assert!(result.is_err());
    }
}
