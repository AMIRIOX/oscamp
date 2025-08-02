#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use core::time::Duration;
use std::os::arceos::api::display::{ax_framebuffer_flush, ax_framebuffer_info, AxDisplayInfo};

const HOR_RES: u32 = 1280;
const VER_RES: u32 = 800;
const DRAW_BUFFER_SIZE: usize = HOR_RES as usize * VER_RES as usize / 10;

static mut DRAW_BUFFER: [u16; DRAW_BUFFER_SIZE] = [0; DRAW_BUFFER_SIZE];
static mut FB_INFO: Option<AxDisplayInfo> = None;

#[no_mangle]
pub extern "C" fn ffs(i: i32) -> i32 {
    if i == 0 {
        0
    } else {
        (i as u32).trailing_zeros() as i32 + 1
    }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let fb_info = ax_framebuffer_info();
    println!(
        "Framebuffer: {}x{}, base_addr=0x{:x}, size={}",
        fb_info.width, fb_info.height, fb_info.fb_base_vaddr, fb_info.fb_size
    );

    unsafe {
        FB_INFO = Some(fb_info);
    }

    println!("Initializing LVGL...");
    lvgl::init();

    unsafe {
        let mut draw_buf = core::mem::MaybeUninit::<lvgl_sys::lv_disp_draw_buf_t>::uninit();
        lvgl_sys::lv_disp_draw_buf_init(
            draw_buf.as_mut_ptr(),
            DRAW_BUFFER.as_mut_ptr() as *mut core::ffi::c_void,
            core::ptr::null_mut(),
            DRAW_BUFFER_SIZE as u32,
        );
        let draw_buf = draw_buf.assume_init();

        let mut disp_drv = core::mem::MaybeUninit::<lvgl_sys::lv_disp_drv_t>::uninit();
        lvgl_sys::lv_disp_drv_init(disp_drv.as_mut_ptr());
        let mut disp_drv = disp_drv.assume_init();

        disp_drv.hor_res = HOR_RES as i16;
        disp_drv.ver_res = VER_RES as i16;
        disp_drv.draw_buf = &draw_buf as *const _ as *mut _;

        extern "C" fn flush_cb(
            disp_drv: *mut lvgl_sys::lv_disp_drv_t,
            area: *const lvgl_sys::lv_area_t,
            color_p: *mut lvgl_sys::lv_color_t,
        ) {
            unsafe {
                let x1 = (*area).x1 as u32;
                let y1 = (*area).y1 as u32;
                let x2 = (*area).x2 as u32;
                let y2 = (*area).y2 as u32;

                if let Some(fb_info) = &FB_INFO {
                    let fb_ptr = fb_info.fb_base_vaddr as *mut u16;

                    for y in y1..=y2 {
                        for x in x1..=x2 {
                            let src_idx = (y - y1) * (x2 - x1 + 1) + (x - x1);
                            let dst_idx = y * HOR_RES + x;

                            let color = *color_p.add(src_idx as usize);
                            *fb_ptr.add(dst_idx as usize) = color.full;
                        }
                    }
                }

                ax_framebuffer_flush();

                lvgl_sys::lv_disp_flush_ready(disp_drv);
            }
        }

        disp_drv.flush_cb = Some(flush_cb);

        println!("Registering display driver...");
        let disp = lvgl_sys::lv_disp_drv_register(&mut disp_drv as *mut _);

        if disp.is_null() {
            panic!("Failed to register display driver");
        }

        println!("Display driver registered successfully!");

        let scr = lvgl_sys::lv_disp_get_scr_act(disp);
        if scr.is_null() {
            panic!("Failed to get screen");
        }

        let blue_color = lvgl_sys::lv_color_t { full: 0x001F };
        lvgl_sys::lv_obj_set_style_bg_color(scr, blue_color, 0);

        let label = lvgl_sys::lv_label_create(scr);
        if !label.is_null() {

            let white_color = lvgl_sys::lv_color_t { full: 0xFFFF };
            lvgl_sys::lv_obj_set_style_text_color(label, white_color, 0);
            
            lvgl_sys::lv_obj_set_style_text_font(label, &lvgl_sys::lv_font_montserrat_48, 0);
            
            let text = b"ArceOS + LVGL Demo!\nSuccessfully Running!\0".as_ptr();
            lvgl_sys::lv_label_set_text(label, text);
            
            lvgl_sys::lv_obj_set_pos(label, (HOR_RES as i16 / 2) - 120, (VER_RES as i16 / 2) - 30);
        }

        println!("UI created, starting main loop...");

        lvgl_sys::lv_obj_invalidate(scr);
        lvgl_sys::lv_refr_now(disp);
    }

    loop {
        unsafe {
            lvgl::tick_inc(Duration::from_millis(5));
            lvgl::task_handler();
        }

        std::thread::sleep(Duration::from_millis(5));
    }
}

