extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use std::thread;
use std::{sync::mpsc, time::Duration};

use nwd::NwgUi;
use std::sync::{Arc, Mutex};
use windows::Win32::{
    Graphics::Gdi::{GetMonitorInfoA, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY},
    UI::WindowsAndMessaging::GetForegroundWindow,
};

use crate::{config, system_provider, watcher};

//use winapi::
// The overlay code is adapted from https://github.com/jcdavis/hroverlay, which is released under the
// Apache 2.0 license (https://raw.githubusercontent.com/jcdavis/hroverlay/main/LICENSE).

#[derive(Default, NwgUi)]
pub struct Overlay {
    #[nwg_control(size: (400, 100), position: (400, 0), flags: "POPUP", ex_flags: winapi::um::winuser::WS_EX_TOPMOST|winapi::um::winuser::WS_EX_LAYERED)]
    #[nwg_events( OnInit: [Overlay::on_init], OnWindowClose: [Overlay::close] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, margin: [0,0,0,0], spacing: 0)]
    layout: nwg::GridLayout,

    #[nwg_resource(family: "Ubuntu", size: 100, weight: 700)]
    font: nwg::Font,

    #[nwg_control(text: "", size: (400, 100), font: Some(&data.font), h_align: HTextAlign::Right, background_color: Some([255, 0, 0]))]
    #[nwg_layout_item(layout: layout, row: 0, col: 0)]
    time_label: nwg::Label,

    #[nwg_control]
    #[nwg_events(OnNotice: [Overlay::on_notice])]
    notice: nwg::Notice,

    text: Arc<Mutex<String>>,
}

impl Overlay {
    fn on_init(&self) {
        use winapi::um::wingdi::RGB;
        use winapi::um::winuser::{SetLayeredWindowAttributes, LWA_COLORKEY};

        let notice = self.notice.sender();
        let (sender, receiver) = mpsc::channel();

        thread::spawn(|| {
            let sysprovider = &system_provider::Win32Provider::new();
            watcher::watch(sysprovider, sender);
        });

        let display_text = self.text.clone();

        thread::spawn(move || {
            for rcv in receiver {
                let cfg = match config::load() {
                    Ok(c) => c,
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                };

                for p in cfg.overlay.show_pattern.iter() {
                    let delay: i32 = *p;
                    let sleep = delay.abs();

                    if delay > 0 {
                        *display_text.lock().unwrap() = rcv.clone();
                    } else {
                        *display_text.lock().unwrap() = "".to_string();
                    }

                    notice.notice();

                    thread::sleep(Duration::from_secs(sleep as u64));
                }

                *display_text.lock().unwrap() = "".to_string();
                notice.notice();
            }
        });

        match self.window.handle {
            nwg::ControlHandle::Hwnd(hwnd) => unsafe {
                SetLayeredWindowAttributes(hwnd, RGB(255, 0, 0), 0, LWA_COLORKEY);
            },
            _ => {
                panic!("Bad handle type for window!")
            }
        }
    }

    fn on_notice(&self) {
        let cfg = match config::load() {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        match self.text.lock().unwrap().as_str() {
            "" => {
                self.window.set_visible(false);
            }
            text => {
                self.time_label.set_text(text);

                let (x, y) = get_position(
                    self.time_label.size(),
                    cfg.overlay.x_offset,
                    cfg.overlay.y_offset,
                );

                self.window.set_position(x, y);
                self.window.set_visible(true);
            }
        }
    }

    fn close(&self) {
        nwg::stop_thread_dispatch()
    }
}

fn get_position(label_sz: (u32, u32), x_offset: u32, y_offset: u32) -> (i32, i32) {
    let mut minf = MONITORINFO::default();
    minf.cbSize = std::mem::size_of::<MONITORINFO>() as _;

    unsafe {
        let hwnd = GetForegroundWindow();
        let hmnt = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
        let _res = GetMonitorInfoA(hmnt, &mut minf as _);
    }

    // get boundaries for top left corner of label
    let lft_boundary = minf.rcMonitor.left;
    let rt_boundary = minf.rcMonitor.right - (label_sz.0 as i32);
    let pos_x = ((rt_boundary - lft_boundary) as f32 * x_offset as f32 / 100_f32).floor() as i32;

    let top_boundary = minf.rcMonitor.top;
    let bottom_boundary = minf.rcMonitor.bottom - (label_sz.1 as i32);
    let pos_y =
        ((bottom_boundary - top_boundary) as f32 * y_offset as f32 / 100_f32).floor() as i32;

    return (pos_x, pos_y);
}
