//! Safe wrappers for GLib calls whose contract the safe bindings cannot express.
//!
//! Every wrapper in this module targets GLib 2.66 or newer. GLib copies borrowed
//! strings during each call, and the wrappers copy every string GLib returns
//! before freeing GLib's own allocation, so no native pointer escapes.

use std::ffi::{CStr, CString, c_char};

/// Returns the host component of a URI as `g_filename_from_uri` reports it.
///
/// GLib assigns the host as soon as it has validated the authority, and only
/// afterwards unescapes the path. A URI whose path holds an invalid escape
/// therefore names a host while yielding no file name at all. The reference
/// reads the host on its own rather than through the conversion's success, so
/// the raw call is needed here; `glib::filename_from_uri` reports the error
/// instead and drops the host.
///
/// An empty host is reported as absent, which is how GLib treats the local form
/// `file:///path`.
pub(crate) fn uri_host(uri: &str) -> Option<String> {
    // GLib reads a NUL-terminated string, so a URI containing NUL cannot be
    // passed on. The reference never sees one, because the strings it
    // classifies arrive from VTE already NUL terminated.
    let uri = CString::new(uri).ok()?;
    let mut host: *mut c_char = std::ptr::null_mut();

    // SAFETY: the URI stays alive and NUL terminated for the whole call, and
    // `host` points at a live pointer slot that GLib clears before it parses.
    // Both the returned file name and the host are owned by this caller.
    let filename =
        unsafe { glib::ffi::g_filename_from_uri(uri.as_ptr(), &mut host, std::ptr::null_mut()) };
    // SAFETY: the file name is either null, which g_free accepts, or an
    // allocation GLib handed over. It is unused because only the host matters.
    unsafe { glib::ffi::g_free(filename.cast()) };

    if host.is_null() {
        return None;
    }

    // SAFETY: a non-null host is a NUL-terminated GLib allocation. The wrapper
    // copies it before handing the allocation back to GLib. GLib validates a
    // host as ASCII labels, so the copy cannot lose bytes.
    let owned = unsafe { CStr::from_ptr(host) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: the host allocation is still owned here and is not used again.
    unsafe { glib::ffi::g_free(host.cast()) };

    Some(owned)
}
