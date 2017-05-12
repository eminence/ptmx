
extern crate libc;

use std::ffi::CStr;

const PTY_DEV: &'static str = "/dev/ptmx";

use std::ptr;
use std::ffi::CString;
use std::io::{Read, Write};

pub enum Fork {
    Parent(MasterPty),
    Child,
}

pub struct MasterPty {
    fd: libc::c_int,
    pub child_pid: libc::pid_t,
}

impl MasterPty {
    pub fn resize(&self, rows: u16, cols: u16) -> std::io::Result<()> {
        let winsize = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let rc = unsafe { libc::ioctl(self.fd, libc::TIOCSWINSZ, &winsize) };

        if rc == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }

    }
}


impl Read for MasterPty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read =
            unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if bytes_read < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(bytes_read as usize)
        }

    }
}

impl Write for MasterPty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) } {
            x if x < 0 => Err(std::io::Error::last_os_error()),
            bytes => Ok(bytes as usize),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ForkError {
    Msg(&'static str),
}

pub fn fork() -> Result<Fork, ForkError> {

    // Initial size
    let winsize = libc::winsize {
        ws_row: 40,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let mut master: libc::c_int = -1;
    let mut slave: libc::c_int = -1;
    if unsafe {
           libc::openpty(&mut master,
                         &mut slave,
                         ptr::null_mut(), // name
                         ptr::null(), // termios
                         &winsize)
       } == -1 {
        return Err(ForkError::Msg("Error in openpty"));
    }

    match unsafe { libc::fork() } {
        -1 => {
            unsafe {
                libc::close(master);
                libc::close(slave);
            }
            return Err(ForkError::Msg("Fork failed"));
        }
        0 => {
            // child

            // forkpty will only close the master fd, but we still need to handle the stdio streams
            unsafe {
                libc::setsid();
                libc::ioctl(slave, libc::TIOCSCTTY, ptr::null() as *const libc::c_void);

                libc::close(master);
                libc::close(libc::STDIN_FILENO);
                libc::close(libc::STDOUT_FILENO);
                libc::close(libc::STDERR_FILENO);

                libc::dup2(slave, libc::STDIN_FILENO);
                libc::dup2(slave, libc::STDOUT_FILENO);
                libc::dup2(slave, libc::STDERR_FILENO);

            }
            return Ok(Fork::Child);
        }
        pid => {
            // parent
            unsafe {
                libc::close(slave);
            }
            return Ok(Fork::Parent(MasterPty {
                                       fd: master,
                                       child_pid: pid,
                                   }));

            //let mut buf = [0u8; 1024];
            //unsafe {
            //    libc::close(slave);
            //    loop {
            //        let nbytes =
            //            libc::read(master, buf.as_mut_ptr() as *mut libc::c_void, buf.len());
            //        if nbytes < 1 {
            //            break;
            //        }
            //        let s = String::from_utf8_lossy(&buf[0..nbytes as usize]);
            //        print!("{}", s);

            //    }
            //}

        }
    }


}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
