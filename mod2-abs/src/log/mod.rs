/*!

# Overview

The `log` module provides logging capabilities with customizable thresholds and log levels. The log level describes
what _kind_ of messages are to be logged, and the numeric threshold is a verbosity level, which describes the
_verbosity_ of the logger.

Here is a simple example.

```
use mod2lib::log::*;

fn main() {
    // Set the global threshold to 1. Any messages logged with greater threshold
    // will not be emitted.
    set_global_logging_threshold(1);

    // Log messages with different channels and levels
    critical!(3, "A critical error occurred!"); // Not emitted
    error!(2, "This is an error message.");     // Not emitted
    warning!(2, "Check your input.");           // Not emitted
    info!(1, "Processing started.");            // Emitted
    debug!(0, "Variable values are correct.");  // Emitted
    trace!(0, "Step through the logic here.");  // Emitted
}
```

## Threshold

The global logging threshold is a numerical value, with higher values meaning more verbose logging. This global value
is the same for all "levels" (info, warning, error, etc.). Individual log entries are logged "at" a given threshold
and are only emitted if their level is _at most_ the global threshold level. In other words, only messages logged at a
threshold less than or equal to the global threshold are emitted. A message with threshold 0 is always emitted.

You can set and get the global logging threshold as follows:

```
use mod2lib::log::{set_global_logging_threshold, get_global_logging_threshold};

// Set the verbosity to 3.
set_global_logging_threshold(3);
println!("The global logging threshold is {}", get_global_logging_threshold());
// Messages logged at levels greater than 3 will not be emitted until the verbosity is set to another value.
// ...
// (Re)set the verbosity to 5.
set_global_logging_threshold(5);
// Messages logged at any nonnegative level will now be emitted from here on.
```

## Levels

Available levels are:  Critical, Error, Warning, Info, Debug, Trace. Messages of a particular level are prefixed with
the (color coded) level name.

# Macros

The following macros are provided for logging at different levels:

 - `critical!`
 - `error!`
 - `warning!`
 - `info!`
 - `debug!`
 - `trace!`

syntax:

```ignore
// With threshold
level!(threshold, "format string", args...);

// Without threshold (indicates threshold of 0, always emitted)
level!("format string", args...);
```

 - `threshold`: An `u8` value representing the threshold for the log message.
 - `"format string"`: A format string, similar to `println!`.
 - `args...`: Arguments to be formatted into the format string.

Examples:

```
use mod2lib::log::{info, set_global_logging_threshold};

fn main() {
    set_global_logging_threshold(3);
    let value = 42;
    // This message will be logged because its threshold (2) <= global threshold (3)
    info!(2, "Processing value: {}", value);
    // This message will not be logged because its threshold (4) > global threshold (3)
    info!(4, "This message will not be logged.");
    // Increase the global threshold
    set_global_logging_threshold(5);
    // Now this message will be logged
    info!(4, "This message will now be logged. The meaning of life, the universe, and everything is {}.", 42);
}
```

# Feature Summary

 - **Threshold Values:** Use thresholds to categorize log messages based on importance or verbosity.
 - Lower threshold values indicate higher importance.
 - The global logging threshold controls which messages are logged.
 - **Default Threshold:** If the threshold argument is omitted in the macro, it defaults to 0.
 - **Dynamic Threshold Adjustment:** Use `set_global_logging_threshold` to change the logging threshold at runtime.
 - **Automatic Logger Initialization:** The logging macros handle logger initialization automatically; no explicit initialization is required.
 - **Thread Safety:** The global logging threshold is managed using atomic operations, ensuring thread safety.

*/
mod formatter;
mod threshold_filter;
mod macros;

use std::sync::{
    atomic::{AtomicU8, Ordering},
    LazyLock
  };

use tracing_subscriber::{
  fmt,
  layer::SubscriberExt,
  Registry
};

use threshold_filter::ThresholdFilterLayer;
use formatter::CustomFieldFormatter;
pub use macros::*;

/// Used for implicit initialization.
static INIT_LOGGER: LazyLock<()> = LazyLock::new(|| {
  let subscriber = Registry::default()
      .with(ThresholdFilterLayer)
      .with(
        fmt::layer()
            .fmt_fields(CustomFieldFormatter)
            .with_target(false)
            // .with_thread_names(true)
            .without_time()
            .with_writer(std::io::stdout),
            // .compact(),
      );

  tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
});

/// This does not need to be called directly. Initializes the logging system.
pub fn init_logger() {
  LazyLock::force(&INIT_LOGGER);
}

///
static GLOBAL_LOGGING_THRESHOLD: AtomicU8 = AtomicU8::new(3); // Default threshold

/// Sets the global threshold before the logger is initialized.
pub fn set_global_logging_threshold(new_threshold: u8) {
  GLOBAL_LOGGING_THRESHOLD.store(new_threshold, Ordering::SeqCst);
}

/// Retrieves the global threshold.
pub fn get_global_logging_threshold() -> u8 {
  GLOBAL_LOGGING_THRESHOLD.load(Ordering::SeqCst)
}


#[cfg(test)]
mod tests {
  use super::*; // Import everything from the parent module
  // use std::sync::atomic::{AtomicU8, Ordering};

  #[test]
  fn test_logging() {
    let foo = 42;

    // Set initial threshold to 3
    set_global_logging_threshold(3);

    // Test that the info message with threshold 2 is logged
    info!(2, "Processing value: {}", foo);
    // This should be logged

    // Test that the debug message with threshold 4 is NOT logged
    debug!(4, "NOT logged Debugging value: {:?}", foo);
    // This should NOT be logged

    // Test that the warning is logged with default threshold of 0
    warning!("An unexpected condition occurred.");
    // This should be logged

    // Test that the error with threshold 5 is NOT logged
    error!(5, "NOT logged An error occurred with value: {}", foo);
    // This should NOT be logged

    // Test that the critical failure with threshold 1 is logged
    critical!(1, "Critical failure: {}", foo);
    // This should be logged

    // Change global threshold to 4
    set_global_logging_threshold(4);

    // Test that the info message with threshold 5 is NOT logged now
    info!(5, "NOT logged This message should now be logged.");
    // This should NOT be logged

    // Change the threshold to a lower value and test again
    set_global_logging_threshold(5);

    info!(5, "This message should now be logged after changing the threshold.");
    // This should be logged
  }
}
