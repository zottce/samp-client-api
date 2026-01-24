use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winnt::LONG;
use winapi::um::winuser::{CallWindowProcA, FindWindowA, SetWindowLongPtrA, GWL_WNDPROC};

use crate::samp::{gamestate, Gamestate};

#[derive(Debug, Clone)]
pub struct WndProcSettings {
    pub callback: fn(),
    pub hwnd: HWND,
}

struct WndProcData {
    callback: fn(),
    hwnd: HWND,
    prev_ptr: LONG,
    additional_cb: Vec<Box<dyn FnMut(UINT, WPARAM, LPARAM) -> bool + 'static>>,
}

static mut WNDPROC_DATA: Option<WndProcData> = None;

pub fn initialize(settings: &WndProcSettings) -> bool {
    if settings.hwnd.is_null() {
        return false;
    }

    if !crate::samp::is_loaded() {
        return false;
    }

    unsafe {
        let prev_ptr = SetWindowLongPtrA(settings.hwnd, GWL_WNDPROC, wndproc as _);

        let data = WndProcData {
            callback: settings.callback,
            hwnd: settings.hwnd,
            additional_cb: Vec::new(),
            prev_ptr,
        };

        WNDPROC_DATA = Some(data);
    }

    return true;
}

pub fn uninitialize() {
    unsafe {
        if let Some(data) = WNDPROC_DATA.take() {
            SetWindowLongPtrA(data.hwnd, GWL_WNDPROC, data.prev_ptr);
        }
    }
}

pub fn append_callback<F>(cb: F)
    where
        F: FnMut(UINT, WPARAM, LPARAM) -> bool + 'static,
{
    unsafe {
        if let Some(data) = WNDPROC_DATA.as_mut() {
            let boxed = Box::new(cb);
            data.additional_cb.push(boxed);
        }
    }
}

extern "system" fn wndproc(wnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if let Some(data) = WNDPROC_DATA.as_mut() {
            (data.callback)();

            for cb in &mut data.additional_cb {
                if cb(msg, wparam, lparam) {
                    return 1;
                }
            }

            return CallWindowProcA(
                Some(std::mem::transmute(data.prev_ptr)),
                wnd,
                msg,
                wparam,
                lparam,
            );
        }

        return 0;
    }
}

pub fn hwnd() -> Option<HWND> {
    unsafe { WNDPROC_DATA.as_ref().map(|data| data.hwnd) }
}
