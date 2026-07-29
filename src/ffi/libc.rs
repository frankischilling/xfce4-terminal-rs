//! Safe wrappers for the small libc surface the port needs.

use std::ffi::{CStr, CString, OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

/// Parses an integer with the same deliberately permissive rules as the C
/// parser.
///
/// The temporary `CString` owns the input for the duration of the call, and
/// libc does not retain its pointer. `strtol` is thread-safe for independent
/// inputs. To preserve the reference behavior, this wrapper ignores the end
/// pointer and `errno`, treats an interior NUL as an empty string, and casts
/// the native result to `i32`. POSIX `strtol` is available on every supported
/// platform.
pub(crate) fn strtol_i32(value: &str) -> i32 {
    let value = CString::new(value).unwrap_or_default();
    unsafe { ::libc::strtol(value.as_ptr(), std::ptr::null_mut(), 0) as i32 }
}

/// Parses an unsigned decimal with the legacy terminalrc conversion rules.
///
/// The temporary `CString` owns the input for the entire `strtoul` call.
/// libc does not retain the borrowed pointer. The wrapper ignores the end
/// pointer and `errno`, then applies the same native-to-`u32` cast as the C
/// preference transform.
pub(crate) fn strtoul_u32(value: &str) -> u32 {
    let value = CString::new(value).unwrap_or_default();
    unsafe { ::libc::strtoul(value.as_ptr(), std::ptr::null_mut(), 10) as u32 }
}

/// Reports whether the process still runs as the user who started it.
///
/// The reference trusts the `SHELL` variable only then, so that a terminal
/// installed set-user-id or set-group-id cannot be told to run something else.
pub(crate) fn privileges_unchanged() -> bool {
    // SAFETY: these four calls read the identity of the calling process, take
    // no arguments, and cannot fail.
    unsafe { ::libc::geteuid() == ::libc::getuid() && ::libc::getegid() == ::libc::getgid() }
}

/// Reports whether a path may be executed, as `access(path, X_OK)` does.
///
/// The answer describes the moment of the call, so it says only that the file
/// looked executable then. The reference accepts the same race: it tests a
/// shell before handing it to VTE, which runs it later.
pub(crate) fn is_executable(path: &OsStr) -> bool {
    let Ok(path) = CString::new(path.as_bytes()) else {
        return false;
    };

    // SAFETY: the path owns its bytes for the whole call and libc does not
    // retain the pointer.
    unsafe { ::libc::access(path.as_ptr(), ::libc::X_OK) == 0 }
}

/// Returns the login shell the password database records for this user.
///
/// `getpwuid` answers from a buffer it owns and reuses, so the wrapper copies
/// the shell before returning and holds no pointer of its own. An entry that
/// does not exist, or that records no shell, is reported as absent.
pub(crate) fn password_database_shell() -> Option<OsString> {
    // SAFETY: the call takes a user id and returns either null or a pointer to
    // its own storage, which stays valid until the next call in this thread.
    let entry = unsafe { ::libc::getpwuid(::libc::getuid()) };
    if entry.is_null() {
        return None;
    }

    // SAFETY: a non-null entry points at an initialized structure whose
    // pw_shell field is either null or a NUL-terminated string in the same
    // storage.
    let shell = unsafe { (*entry).pw_shell };
    if shell.is_null() {
        return None;
    }

    // SAFETY: the shell is NUL terminated and stays valid for this copy.
    Some(OsString::from_vec(
        unsafe { CStr::from_ptr(shell) }.to_bytes().to_vec(),
    ))
}

/// Parses a legacy terminalrc floating-point value.
///
/// The C preferences first use the active C locale. If that leaves trailing
/// input, they retry with GLib's locale-independent ASCII parser. Both calls
/// borrow the temporary `CString` only for the duration of this wrapper.
pub(crate) fn terminalrc_double(value: &str) -> f64 {
    let value = CString::new(value).unwrap_or_default();
    let mut end = std::ptr::null_mut();
    let parsed = unsafe { ::libc::strtod(value.as_ptr(), &mut end) };
    if end.is_null() || unsafe { *end } == 0 {
        parsed
    } else {
        unsafe { glib::ffi::g_ascii_strtod(value.as_ptr(), std::ptr::null_mut()) }
    }
}
