#![deny(unused_must_use, unsafe_op_in_unsafe_fn)]

#[cfg(unix)]
mod unix {
    #[allow(unused)]
    use std::ffi::{CStr, OsString};
    use std::io::Error;
    #[allow(unused)]
    use std::os::raw::{c_char, c_int, c_void};
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;
    #[allow(unused)]
    use std::ptr::NonNull;

    #[cfg(unix)]
    #[allow(unused)]
    fn from_link(link: &CStr) -> Result<PathBuf, Error> {
        let mut buf = Vec::with_capacity(libc::PATH_MAX as _);
        let len = match unsafe {
            libc::readlink(
                link.as_ptr(),
                buf.as_mut_ptr() as *mut c_char,
                libc::PATH_MAX as _,
            )
        } {
            -1 => return Err(Error::last_os_error()),
            n => n as usize,
        };
        unsafe {
            buf.set_len(len);
        }
        buf.shrink_to_fit();
        Ok(OsString::from_vec(buf).into())
    }

    #[cfg(target_os = "linux")]
    pub fn exec_path() -> Result<PathBuf, Error> {
        from_link(CStr::from_bytes_with_nul(b"/proc/self/exe\0").unwrap())
    }

    #[cfg(target_os = "dragonfly")]
    pub fn exec_path() -> Result<PathBuf, Error> {
        from_link(CStr::from_bytes_with_nul(b"/proc/curproc/file\0").unwrap())
    }

    #[cfg(target_os = "netbsd")]
    pub fn exec_path() -> Result<PathBuf, Error> {
        from_link(CStr::from_bytes_with_nul(b"/proc/curproc/exe\0").unwrap())
    }

    #[cfg(target_os = "freebsd")]
    pub fn exec_path() -> Result<PathBuf, Error> {
        let mib = [
            libc::CTL_KERN,
            libc::KERN_PROC,
            libc::KERN_PROC_PATHNAME,
            -1,
        ];
        let mut buf = Vec::with_capacity(libc::PATH_MAX as _);
        let mut len = libc::PATH_MAX as libc::size_t;
        if unsafe {
            libc::sysctl(
                &mib as *const c_int,
                4,
                buf.as_mut_ptr() as *mut c_void,
                &mut len as *mut libc::size_t,
                std::ptr::null(),
                0,
            )
        } == -1
        {
            return Err(Error::last_os_error());
        }
        unsafe {
            buf.set_len(len as usize);
        }
        buf.shrink_to_fit();
        Ok(OsString::from_vec(buf).into())
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    pub fn exec_path() -> Result<PathBuf, Error> {
        let mut buf = Vec::with_capacity(libc::PATH_MAX as _);
        let mut len = libc::PATH_MAX as u32;
        if unsafe {
            libc::_NSGetExecutablePath(buf.as_mut_ptr() as *mut c_char, &mut len as *mut u32)
        } == -1
        {
            return Err(Error::last_os_error());
        }
        unsafe {
            buf.set_len(len as usize);
        }
        buf.shrink_to_fit();
        Ok(OsString::from_vec(buf).into())
    }

    #[cfg(any(target_os = "illumos", target_os = "solaris"))]
    pub fn exec_path() -> Result<PathBuf, Error> {
        let p = unsafe { libc::getexecname() as *mut c_char };
        let p = NonNull::new(p)
            .ok_or_else(|| Error::last_os_error())?
            .as_ptr();
        let buf = unsafe { CStr::from_ptr(p).to_bytes().to_owned() };
        Ok(OsString::from_vec(buf).into())
    }
}

#[cfg(unix)]
pub use self::unix::exec_path;

#[cfg(windows)]
mod windows {
    use std::ffi::OsString;
    use std::io::Error;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;
    use winapi::um::libloaderapi::GetModuleFileNameW;

    pub fn exec_path() -> Result<PathBuf, Error> {
        // this approach adapted from how std does getcwd
        let mut cap = 512;
        let mut buf: Vec<u16> = Vec::with_capacity(cap as usize);
        let len = loop {
            match unsafe { GetModuleFileNameW(std::ptr::null_mut(), buf.as_mut_ptr(), cap) } {
                0 => return Err(Error::last_os_error()), // paths can't be empty
                n if n < cap => break n,
                _ => {
                    buf.reserve(cap);
                    cap *= 2;
                }
            }
        };
        unsafe {
            buf.set_len(len as usize);
        }
        Ok(OsString::from_wide(&buf).into())
    }
}

#[cfg(windows)]
pub use self::windows::exec_path;
