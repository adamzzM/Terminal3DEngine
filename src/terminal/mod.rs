

#[cfg(unix)]
pub fn get_terminal_size() -> (u16, u16) {
    use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
    unsafe {
        let mut ws: winsize = std::mem::zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws);
        (ws.ws_col, ws.ws_row)
    }
}

#[cfg(windows)]
pub fn get_terminal_size() -> (u16, u16) {
    use winapi::um::wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO};
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;
    use std::mem::zeroed;
    unsafe {
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        GetConsoleScreenBufferInfo(handle, &mut csbi);
        let width = (csbi.srWindow.Right - csbi.srWindow.Left + 1) as u16;
        let height = (csbi.srWindow.Bottom - csbi.srWindow.Top + 1) as u16;
        (width, height)
    }
}