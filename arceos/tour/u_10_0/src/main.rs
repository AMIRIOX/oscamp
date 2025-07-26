#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::os::arceos::api::display::{ax_framebuffer_flush, ax_framebuffer_info, AxDisplayInfo};
use std::os::arceos::modules::axdriver;
use std::thread::sleep;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct RGBA {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

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

struct Graphic<'screen_exist> {
    frame_buffer: &'screen_exist mut [u8],
    height: u32,
    width: u32,
}

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

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    //axdriver::init_drivers();
    let fb_info = ax_framebuffer_info();

    let mut screen = Graphic::new(&fb_info);

    screen.clear_screen(0xFFFFFF);
    ax_framebuffer_flush();

    screen.draw_rect(0x80C0FF, (500, 200), 400, 300);
    ax_framebuffer_flush();

    screen.draw_line(0x80C0FF, (25, 25), (1000, 500));
    ax_framebuffer_flush();

    screen.draw_circle(0x80C0FF, (512, 384), 100);
    ax_framebuffer_flush();

    sleep(core::time::Duration::new(2, 0));
}
