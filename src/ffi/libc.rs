//! Safe wrappers for the small libc surface used by the option parser.

use std::ffi::CString;

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
pub(crate) fn strtoul_u32(value: &str) -> u32 {
    let value = CString::new(value).unwrap_or_default();
    unsafe { ::libc::strtoul(value.as_ptr(), std::ptr::null_mut(), 10) as u32 }
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
