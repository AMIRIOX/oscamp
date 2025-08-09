pub use axdriver_input::{InputEvent, InputEventType, MouseEvent, TabletEvent, ProcessedInputEvent, RelativeAxis, AbsoluteAxis, InputDriverOps};
use alloc::string::{String, ToString};

/// Poll for input events from the global input devices.
/// Returns Some(event) if an event is available, or None if no events.
pub fn ax_input_poll_event() -> Option<InputEvent> {
    axinput::poll_event()
}

/// Check if any input events are available without consuming them.
pub fn ax_input_has_events() -> bool {
    axinput::has_events()
}

/// Get information about the first available input device.
pub fn ax_input_device_info() -> Option<String> {
    Some(axinput::device_info())
}

/// Get the name of the first available input device.
pub fn ax_input_device_name() -> Option<String> {
    Some(axinput::device_name())
}