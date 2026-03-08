// use core::fmt::Formatter;

// use embedded_hal::i2c::{I2c, SevenBitAddress};
// use embedded_hal::i2c::{Error as I2cError, ErrorKind as I2cErrorKind};

/// All possible errors
/// Display not implemented for no_std support
#[derive(Clone, Copy, Debug)]
pub enum Error<E> {
    CommandError,  /// AS3935 not happy with the command received
    NotReadyForCommand, /// AS3935 not ready to received a command

    /// Oscillator failed to calibrate
    OscFailedCalibration,
    /// response data error
    ResponseError,
    /// Value too big
    ValueLimit,
    /// An error in the  underlying I²C system
    I2c(E),
}


