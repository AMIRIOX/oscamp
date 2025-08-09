#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::os::arceos::api::display::{ax_framebuffer_flush, ax_framebuffer_info, AxDisplayInfo};
use std::os::arceos::api::input::{
    ax_input_device_info, ax_input_device_name, ax_input_has_events, ax_input_poll_event,
    AbsoluteAxis, InputEvent, InputEventType, RelativeAxis,
};
use std::os::arceos::modules::axdriver;

#[cfg(feature = "display")]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RGBA {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

#[cfg(feature = "display")]
impl RGBA {
    fn new(color: u32) -> Self {
        RGBA {
            red: (color & 0xFF) as u8,
            green: ((color >> 8) & 0xFF) as u8,
            blue: ((color >> 16) & 0xFF) as u8,
            alpha: 0xFF as u8,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

#[cfg(feature = "display")]
struct Graphic<'screen_exist> {
    frame_buffer: &'screen_exist mut [u8],
    height: u32,
    width: u32,
}

#[cfg(feature = "display")]
impl Graphic<'_> {
    fn new(fb_info: &AxDisplayInfo) -> Self {
        Graphic {
            frame_buffer: unsafe {
                core::slice::from_raw_parts_mut(fb_info.fb_base_vaddr as *mut u8, fb_info.fb_size)
            },
            height: fb_info.height,
            width: fb_info.width,
        }
    }
    fn fill_single_pixel(&mut self, color: u32, pos: (u32, u32)) {
        let x = pos.0;
        let y = pos.1;
        if x >= 0 && y >= 0 && x < self.width && y < self.height {
            let pixel_offset = ((y * self.width + x) * 4) as usize;
            let color = RGBA::new(color);
            self.frame_buffer[pixel_offset..pixel_offset + 4].copy_from_slice(color.as_bytes());
        }
    }

    fn clear_screen(&mut self, color: u32) {
        let color = RGBA::new(color);

        for chunk in self.frame_buffer.chunks_exact_mut(4) {
            chunk.copy_from_slice(color.as_bytes());
        }
    }

    fn draw_line(&mut self, color: u32, pos1: (u32, u32), pos2: (u32, u32)) {
        let mut xi = pos1.0 as i32;
        let mut yi = pos1.1 as i32;

        let x2 = pos2.0 as i32;
        let y2 = pos2.1 as i32;

        let sx = if xi < x2 { 1 } else { -1 };
        let sy = if yi < y2 { 1 } else { -1 };

        let dx = (xi - x2).abs();
        let dy = -((yi - y2).abs());

        let mut err = dx + dy;

        loop {
            self.fill_single_pixel(color, (xi as u32, yi as u32));

            if xi == x2 && yi == y2 {
                break;
            }

            let e2 = err * 2;

            if e2 >= dy {
                err += dy;
                xi += sx;
            }

            if e2 <= dx {
                err += dx;
                yi += sy;
            }
        }
    }

    fn draw_rect(&mut self, color: u32, pos: (u32, u32), h: u32, w: u32) {
        self.draw_line(color, pos, (pos.0 + h, pos.1));
        self.draw_line(color, (pos.0, pos.1 + w), (pos.0 + h, pos.1 + w));

        // (x, y)       --    (x + h, y)
        //   |                      |
        // (x, y + w)   --    (x + h, y + w)

        self.draw_line(color, pos, (pos.0, pos.1 + w));
        self.draw_line(color, (pos.0 + h, pos.1), (pos.0 + h, pos.1 + w));
    }

    fn draw_block(&mut self, color: u32, pos: (u32, u32), h: u32, w: u32) {
        let x = pos.0;
        let y = pos.1;
        for xi in x..x + h {
            for yi in y..y + w {
                self.fill_single_pixel(color, (xi, yi));
            }
        }
    }

    fn draw_circle_octants(&mut self, x0: u32, y0: u32, x: i32, y: i32, color: u32) {
        self.fill_single_pixel(color, ((x0 as i32 + x) as u32, (y0 as i32 + y) as u32));
        self.fill_single_pixel(color, ((x0 as i32 - x) as u32, (y0 as i32 + y) as u32));
        self.fill_single_pixel(color, ((x0 as i32 + x) as u32, (y0 as i32 - y) as u32));
        self.fill_single_pixel(color, ((x0 as i32 - x) as u32, (y0 as i32 - y) as u32));
        self.fill_single_pixel(color, ((x0 as i32 + y) as u32, (y0 as i32 + x) as u32));
        self.fill_single_pixel(color, ((x0 as i32 - y) as u32, (y0 as i32 + x) as u32));
        self.fill_single_pixel(color, ((x0 as i32 + y) as u32, (y0 as i32 - x) as u32));
        self.fill_single_pixel(color, ((x0 as i32 - y) as u32, (y0 as i32 - x) as u32));
    }

    /*
     * p = (x + 1)^2 + (y - 0.5)^2
     * */
    fn draw_circle(&mut self, color: u32, pos: (u32, u32), rd: u32) {
        let mut xi = 0 as i32;
        let mut yi = rd as i32;
        let mut p = 1 - rd as i32;

        self.draw_circle_octants(pos.0, pos.1, xi, yi, color);
        while xi < yi {
            xi += 1;

            if p <= 0 {
                p = p + 2 * xi + 1;
            } else {
                yi -= 1;
                p = p + 2 * xi - 2 * yi + 1;
            }

            self.draw_circle_octants(pos.0, pos.1, xi, yi, color);
        }
    }
}

pub fn input_device_info() {
    if let Some(device_name) = ax_input_device_name() {
        println!("Input device: {}", device_name);

        if let Some(device_info) = ax_input_device_info() {
            println!("Device info: {}", device_info);
        }
    } else {
        println!("No input devices found");
        return;
    }
}

pub fn input_polling_loop(max_loops: usize, mut screen: Option<&mut Graphic>) {
    input_device_info();
    println!("Starting input polling loop...");

    let mut x = 0u32;
    let mut y = 0u32;
    let mut loop_count = 0;
    let mut pressed = false;

    while loop_count < max_loops {
        if ax_input_has_events() {
            //print!("Input events available: ");

            if let Some(event) = ax_input_poll_event() {
                match event.get_type() {
                    Some(InputEventType::Relative) => {
                        if event.code == RelativeAxis::X as u16 {
                            panic!();
                            let mouse_dx = event.value; // as i32;
                            x += mouse_dx;
                            // println!("Quick mouse X movement: {}", mouse_dx);
                        } else if event.code == RelativeAxis::Y as u16 {
                            let mouse_dy = event.value; // as i32;
                            y += mouse_dy;
                            // println!("Quick mouse Y movement: {}", mouse_dy);
                        }
                    }
                    Some(InputEventType::Absolute) => {
                        if event.code == AbsoluteAxis::X as u16 {
                            let tablet_x = event.value;
                            x = tablet_x;
                            // println!("Quick tablet X position: {}", tablet_x);
                        } else if event.code == AbsoluteAxis::Y as u16 {
                            let tablet_y = event.value;
                            y = tablet_y;
                            // println!("Quick tablet Y position: {}", tablet_y);
                        }
                    }
                    Some(InputEventType::Key) => {
                        pressed = event.value == 1;
                    }
                    Some(InputEventType::Sync) => {
                        if let Some(ref mut screen) = screen {
                            let (h, w) = (screen.height, screen.width);
                            let ax = (x as u64 * (w as u64 - 1) + 16383) / 32767;
                            let ay = (y as u64 * (h as u64 - 1) + 16383) / 32767;

                            if pressed {
                                screen.draw_line(
                                    0x006400,
                                    (0, 0),
                                    (ax.try_into().unwrap(), ay.try_into().unwrap()),
                                );
                                ax_framebuffer_flush();
                                println!("Moved to: ({}, {})", ax, ay);
                            }
                        } else {
                            if pressed {
                                println!("Moved to: ({}, {})", x, y);
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // println!("No fucking events.");
            // yield?
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        loop_count += 1;
    }

    println!(
        "Input polling loop completed after {} iterations",
        loop_count
    );
}

#[no_mangle]
pub extern "C" fn main() {
    let fb_info = ax_framebuffer_info();
    let mut screen = Graphic::new(&fb_info);
    screen.clear_screen(0xFFB6C1);
    ax_framebuffer_flush();

    input_polling_loop(20000, Some(&mut screen));
    std::thread::sleep(core::time::Duration::new(2, 0));
    std::process::exit(0);
}
