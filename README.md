### Problem

Since `window_proc()` function of `win_win::WindowProc` trait takes `&self`,
mutable state has to be wrapped in `RefCell<>`.

In `window_proc()` we use `.borrow_mut()` to access it.

However, if we forget to release this borrow when calling Windows API functions that cause window proc reentrancy (e.g. `SendMessage()` or `MessageBox()`),
second invocation of `window_proc()` will fail with a runtime error "already borrowed".

This failure to account for reentrancy could be easily missed by manual testing if `.borrow_mut()` only happens on some paths in `window_proc()`.

It would be nice to have some kind of compile-time enforcement.

### Solution

Instead of passing `RefCell<State>` to `window_proc()`, we will pass some opaque type `StateRef<State>` with the following public interface:

```rust
impl StateRef<State> {
    pub fn state_mut(&mut self) -> impl DerefMut<Target=State>;
    pub fn reent<'a>(&'a mut self) -> Reent<'a>;
}
```

`Reent` is a zero-sized type that proves that we are not holding mut borrow of `State` at the moment.

Windows API calls that are known to cause reentrancy should have safe wrappers
that take a `Reent` argument:

```rust
pub fn message_box(
    _reent: Reent,
    hwnd: HWND,
    title: &str,
    message: &str,
    u_type: UINT,
) -> c_int {
    unsafe {
        MessageBoxW(
            hwnd,
            message.to_wide_null().as_ptr(),
            title.to_wide_null().as_ptr(),
            u_type)
    }
}
```

Usage example:

```rust
fn window_proc(
    sr: &mut StateRef<State>,
    hwnd: HWND, msg: UINT, _wparam: WPARAM, _lparam: LPARAM,
)-> Option<LRESULT> {
    let mut state = sr.state_mut();
    // do stuff with state

    if msg == WM_LBUTTONDOWN {
        drop(state);  // Release state borrow.
                      // Borrow checker will compain if we forget to do it.

        message_box(sr.reent(), hwnd, "hello", "title", MB_OK);

        let mut state = sr.state_mut();  // borrow it again if needed
        // do stuff
    }
    None
}
```.