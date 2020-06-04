use wio::wide::ToWide;

use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::*;
use winapi::ctypes::c_int;

mod win_win_reent;
use win_win_reent::{Reent, StateRef, WindowProcState, OuterState};

pub fn message_box(
    _reent: Reent,
    hwnd: HWND,
    title: &str,
    message: &str,
    u_type: UINT,
) -> c_int {
    let res = unsafe {
        MessageBoxW(
            hwnd,
            message.to_wide_null().as_ptr(),
            title.to_wide_null().as_ptr(),
            u_type)
    };
    assert!(res != 0, "{}", std::io::Error::last_os_error());
    res
}

struct State(i32);

impl WindowProcState for State {
    fn window_proc(
        sr: &mut StateRef<Self>,
        hwnd: HWND, msg: UINT, _wparam: WPARAM, _lparam: LPARAM,
    )-> Option<LRESULT> {
        let mut state = sr.state_mut();
        state.0 += 1;
        if msg == WM_DESTROY {
            unsafe {
                PostQuitMessage(0);
            }
        }
        if msg == WM_LBUTTONDOWN {
            drop(state);  // Release state borrow.
                          // Borrow checker will compain if we forget to do it.
            message_box(sr.reent(), hwnd, "hello", "title", MB_OK);

            let mut state = sr.state_mut();
            state.0 += 1;
        }
        None
    }
}

fn main() {
    let state = State(0);
    let state = OuterState::new(state);
    unsafe {
        let cursor = LoadCursorW(0 as HINSTANCE, IDC_ARROW);
        let win_class = win_win::WindowClass::builder("example")
            .cursor(cursor)
            .build().unwrap();
        let hwnd = win_win::WindowBuilder::new(state, &win_class)
            .name("zzz")
            .style(WS_OVERLAPPEDWINDOW)
            .build();
        ShowWindow(hwnd, SW_SHOWNORMAL);
        win_win::runloop(std::ptr::null_mut());
    }
}
