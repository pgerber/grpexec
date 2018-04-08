#[macro_use]
extern crate derive_error;
extern crate libc;
#[macro_use]
extern crate log;

use std::ffi::{CStr, CString};
use std::ptr;

#[derive(Debug, Error)]
pub enum GrpError {
    #[error(msg_embedded, no_from, non_std)]
    CError(String),

    /// not a member of group
    #[error(no_from, non_std)]
    NotAGroupMember,

    #[error(no_from, non_std)]
    NoSuchGroup,
}

impl GrpError {
    unsafe fn from_errno(prefix: &str) -> GrpError {
        let mut buf = [0_u8; 80];
        libc::strerror_r(errno(), buf.as_mut_ptr() as *mut libc::c_char, buf.len());
        let msg = CStr::from_ptr(buf.as_ptr() as *const libc::c_char);
        GrpError::CError(format!("{}: {}", prefix, msg.to_string_lossy()))
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Uid(libc::uid_t);

#[derive(Clone, Copy, PartialEq)]
struct Gid(libc::gid_t);


#[inline]
unsafe fn errno() -> libc::c_int {
    *libc::__errno_location()
}

#[inline]
fn get_gid() -> Gid {
    unsafe { Gid(libc::getgid()) }
}

fn get_gid_by_name(name: &str) -> Result<Gid, GrpError> {
    debug!("searching for GID of {:?}", name);
    let c_name = match CString::new(name) {
        Ok(n) => n,
        Err(_) => {
            warn!("failed to convert {:?} to a CString", name);
            return Err(GrpError::NoSuchGroup);
        }
    };

    unsafe {
        let mut group = libc::group {
            gr_name: ptr::null_mut(),
            gr_passwd: ptr::null_mut(),
            gr_gid: 65534,
            gr_mem: ptr::null_mut(),
        };
        let mut result = ptr::null_mut();
        let mut buf = vec![0_u8; 128];
        loop {
            match libc::getgrnam_r(
                c_name.as_ptr(),
                &mut group,
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                &mut result,
            ) {
                0 => break, // success
                -1 if errno() == libc::ERANGE => {
                    // buffer too small
                    let new_len = buf.len() * 2;
                    buf.resize(new_len, 0);
                },
                _ => return Err(GrpError::from_errno("failed to get GID by name")),
            }
        }

        if let Some(group) = result.as_ref() {
            debug!("GID of {:?} is {}", name, group.gr_gid);
            Ok(Gid(group.gr_gid))
        } else {
            debug!("group {:?} not found", name);
            Err(GrpError::NoSuchGroup)
        }
    }
}

fn is_user_in_group(gid: Gid) -> Result<bool, GrpError> {
    if get_gid() == gid {
        debug!("current GID equals target GID");
        return Ok(true);
    }
    let size = unsafe { libc::getgroups(0, ptr::null_mut()) };
    assert!(size >= 0);
    let mut groups = vec![65534 as libc::gid_t; size as usize];
    unsafe {
        match libc::getgroups(size, groups.as_mut_ptr()) {
            -1 => Err(GrpError::from_errno("failed to fetch list of groups")),
            ret => {
                if size > ret {
                    groups.truncate(ret as usize);
                }
                Ok(groups.contains(&gid.0))
            }
        }
    }
}

fn set_gid(gid: Gid) -> Result<(), GrpError> {
    debug!("setting GID to {}", gid.0);
    unsafe {
        match libc::setgid(gid.0) {
            0 => Ok(()),
            -1 => Err(GrpError::from_errno("failed to set new GID")),
            _ => unreachable!(),
        }
    }
}

fn drop_privileges() -> Result<(), GrpError> {
    unsafe {
        match libc::setuid(libc::getuid()) {
            0 => Ok(()),
            -1 => Err(GrpError::from_errno("failed to drop privileges")),
            _ => unreachable!(),
        }
    }
}

/// change group then drop privileges
///
/// Set process group to `group` if the user is part of that group. Once
/// this succeeded, privileges are dropped (i.e. EUID is reset to UID).
///
/// An error is returned if the user is not part of `group` or if setting
/// the group fails.
pub fn drop_privileges_with_group(group: &str) -> Result<(), GrpError> {
    let gid = get_gid_by_name(group)?;
    if is_user_in_group(gid)? {
        set_gid(gid)?;
        drop_privileges()
    } else {
        Err(GrpError::NotAGroupMember)
    }
}
