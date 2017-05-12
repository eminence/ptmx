extern crate ptmx;
extern crate libc;

use std::ffi::{CStr, CString};
use std::ptr;
use std::io::{Read, Write, stdout};
use std::time::{Duration, Instant};


fn main() {


    match ptmx::fork().unwrap() {
        ptmx::Fork::Parent(mut master) => {

            let mut buf = [0; 1024];
            let start = Instant::now();
            let mut did_resize = false;
            let mut did_c = false;
            let mut did_q = false;

            loop {
                let so = stdout();
                match master.read(&mut buf) {
                    Err(..) => break,
                    Ok(0) => break,
                    Ok(nbytes) => {
                        so.lock().write_all(&buf[0..nbytes]).unwrap();

                        if !did_resize && start.elapsed() > Duration::from_secs(5) {
                            master.resize(60, 160).unwrap();
                            println!("Resizing");
                            did_resize = true;
                        }
                        if !did_c && start.elapsed() > Duration::from_secs(10) {
                            master.write("c".as_bytes());
                            did_c = true;
                        }
                        if !did_q && start.elapsed() > Duration::from_secs(15) {
                            master.write("q".as_bytes());
                            did_q = true;
                        }
                    }
                }

            }
        }
        ptmx::Fork::Child => {

            let cmd = CString::new("top").unwrap();
            let a = CString::new("-d").unwrap();
            let b = CString::new("1").unwrap();
            let args = [cmd.as_ptr(), a.as_ptr(), b.as_ptr(), ptr::null()];
            unsafe {
                libc::execvp(cmd.as_ptr(), args.as_ptr());
            }
        }
    }


}
