//! [ArceOS](https://github.com/arceos-org/arceos) input module.
//!
//! Provides input device management using a singleton pattern for consistent state management.

#![no_std]

extern crate alloc;
use alloc::string::{String, ToString};

#[macro_use]
extern crate log;

#[doc(no_inline)]
pub use axdriver_input::{InputEvent, InputEventType, MouseEvent, TabletEvent, ProcessedInputEvent, RelativeAxis, AbsoluteAxis, InputDriverOps};

use axdriver::{prelude::*, AxDeviceContainer};
use axdriver_input::BaseDriverOps;
use axsync::Mutex;
use lazyinit::LazyInit;

static MAIN_INPUT: LazyInit<Mutex<AxInputDevice>> = LazyInit::new();

/// Initializes the input subsystem by underlayer devices.
pub fn init_input(mut input_devs: AxDeviceContainer<AxInputDevice>) {
    info!("Initialize input subsystem...");

    let dev = input_devs.take_one().expect("No input device found!");
    info!("  use input device 0: {:?}", dev.device_name());
    MAIN_INPUT.init_once(Mutex::new(dev));
}

/// Poll for input events from the main input device.
/// Returns Some(event) if an event is available, or None if no events.
pub fn poll_event() -> Option<InputEvent> {
    match MAIN_INPUT.lock().poll_event() {
        Ok(event) => Some(event),
        Err(_) => None,
    }
}

/// Check if any input events are available without consuming them.
pub fn has_events() -> bool {
    MAIN_INPUT.lock().has_events()
}

/// Get information about the main input device.
pub fn device_info() -> String {
    MAIN_INPUT.lock().device_info().to_string()
}

/// Get the name of the main input device.
pub fn device_name() -> String {
    MAIN_INPUT.lock().device_name().to_string()
}