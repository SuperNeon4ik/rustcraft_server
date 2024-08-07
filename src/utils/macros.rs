#[macro_export]
macro_rules! log {
    (error, $($arg:tt)*) => {
        LOGGER.error(file!(), &format!($($arg)*))
    };
    (warn, $($arg:tt)*) => {
        LOGGER.warn(file!(), &format!($($arg)*))
    };
    (info, $($arg:tt)*) => {
        LOGGER.info(file!(), &format!($($arg)*))
    };
    (verbose, $($arg:tt)*) => {
        LOGGER.verbose(file!(), &format!($($arg)*))
    };
    (debug, $($arg:tt)*) => {
        LOGGER.debug(file!(), &format!($($arg)*))
    };
}