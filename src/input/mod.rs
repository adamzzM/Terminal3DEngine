
#[cfg(windows)]
use winapi::um::wincon::*;
use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::STD_INPUT_HANDLE;
use winapi::um::consoleapi::ReadConsoleInputW;  // Add this line
use std::mem::zeroed;


#[cfg(windows)]
pub fn poll_key() -> Option<u8> {
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);

        let mut rec: INPUT_RECORD = zeroed();
        let mut read = 0;

        let mut last_key: Option<u8> = None;

        loop {
            if PeekConsoleInputW(handle, &mut rec, 1, &mut read) == 0 || read == 0 {
                break;
            }

            ReadConsoleInputW(handle, &mut rec, 1, &mut read);

            if rec.EventType == KEY_EVENT {
                let key = rec.Event.KeyEvent();
                if key.bKeyDown != 0 {
                    let ch = key.uChar.AsciiChar();
                    if *ch != 0 {
                        last_key = Some(*ch as u8);
                    }
                }
            }
        }

        last_key
    }
}


// still untensted

#[cfg(unix)]
mod input {
    use libc::*;
    use std::mem;
    use std::os::unix::io::AsRawFd;

    static mut ORIGINAL: termios = unsafe { mem::zeroed() };

    pub fn init() {
        unsafe {
            let fd = std::io::stdin().as_raw_fd();
            tcgetattr(fd, &mut ORIGINAL);

            let mut raw = ORIGINAL;
            raw.c_lflag &= !(ICANON | ECHO);
            raw.c_cc[VMIN] = 0;
            raw.c_cc[VTIME] = 0;

            tcsetattr(fd, TCSANOW, &raw);
        }
    }

    pub fn shutdown() {
        unsafe {
            let fd = std::io::stdin().as_raw_fd();
            tcsetattr(fd, TCSANOW, &ORIGINAL);
        }
    }

    pub fn poll_key() -> Option<u8> {
        let mut buf = [0u8; 1];
        match std::io::stdin().read(&mut buf) {
            Ok(1) => Some(buf[0]),
            _ => None,
        }
    }
}
