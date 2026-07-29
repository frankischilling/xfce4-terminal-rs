//! Safe wrappers for GLib calls whose contract the safe bindings cannot express.
//!
//! Every wrapper in this module targets GLib 2.66 or newer. GLib copies borrowed
//! strings during each call, and the wrappers copy every string GLib returns
//! before freeing GLib's own allocation, so no native pointer escapes.

use std::ffi::{CStr, CString, OsStr, OsString, c_char};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

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

/// Returns the last component of a path as `g_path_get_basename` reports it.
///
/// This is not the base name Rust's path types report. GLib ignores trailing
/// separators, so `/bin/` yields `bin`, and it never yields nothing: the root
/// directory is its own base name and an empty path yields the current
/// directory. The reference derives a shell's argument zero this way, so the
/// port has to agree on all three.
///
/// A path holding an interior NUL cannot reach GLib and is returned unchanged.
/// The reference cannot meet one, because a path it asks about came either from
/// the environment or from splitting a command line, and both are NUL
/// terminated already.
pub(crate) fn path_basename(path: &OsStr) -> OsString {
    let Ok(owned) = CString::new(path.as_bytes()) else {
        return path.to_os_string();
    };

    // SAFETY: the path stays alive and NUL terminated for the whole call, and
    // GLib copies what it reads. The returned base name is owned by this
    // caller.
    let basename = unsafe { glib::ffi::g_path_get_basename(owned.as_ptr()) };
    // SAFETY: GLib returns a non-null NUL-terminated allocation for any input.
    // The wrapper copies its bytes before handing the allocation back.
    let bytes = unsafe { CStr::from_ptr(basename) }.to_bytes().to_vec();
    // SAFETY: the allocation is still owned here and is not used again.
    unsafe { glib::ffi::g_free(basename.cast()) };

    OsString::from_vec(bytes)
}
