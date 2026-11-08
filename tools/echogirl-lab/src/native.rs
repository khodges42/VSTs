use std::error::Error;
// Unsafe code is limited to the development host's OLE apartment and Windows
// message pump. The plugin and DSP core forbid handwritten unsafe code.
#[cfg(windows)]
#[link(name = "ole32")]
unsafe extern "system" {
    fn OleInitialize(p: *mut std::ffi::c_void) -> i32;
    fn OleUninitialize();
}
pub struct Ole;
impl Ole {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        #[cfg(windows)]
        if unsafe { OleInitialize(std::ptr::null_mut()) } < 0 {
            return Err("Cannot initialize OLE STA".into());
        }
        Ok(Self)
    }
}
impl Drop for Ole {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            OleUninitialize();
        }
    }
}
pub fn pump() {
    #[cfg(windows)]
    // All messages are owned by this UI thread; no plugin lock is held while
    // dispatching callbacks, and MSG storage outlives each native call.
    unsafe {
        use winapi::um::winuser::*;
        let mut msg = std::mem::zeroed();
        for _ in 0..1024 {
            if PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) == 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
