//! Safe wrappers for GTK accelerator-map persistence.
//!
//! Every wrapper in this module targets GTK 3.24 or newer and requires GTK to
//! be initialized on the calling main thread. GTK copies borrowed strings
//! during each call, and no native pointer escapes. Functions without a native
//! error channel are treated as successful once their Rust arguments pass
//! validation; boolean GTK results are returned to the caller.

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use glib::translate::{FromGlib, IntoGlib};

/// Loads a file produced by `gtk_accel_map_save`.
///
/// GTK owns the global accelerator map and does not retain the filename. The
/// C function is process-global and must run after GTK initialization on the
/// GTK main thread, which this wrapper checks before making the call.
pub(crate) fn load_accel_map(path: &Path) -> Result<(), String> {
    ensure_main_thread()?;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| "accelerator path contains NUL".to_owned())?;
    unsafe { gtk::ffi::gtk_accel_map_load(path.as_ptr()) };
    Ok(())
}

/// Saves GTK's global accelerator map in its native compatible format.
///
/// GTK reads the filename during the call and does not retain its pointer, so
/// the temporary `CString` may be dropped on return. The map is process-global
/// and may only be saved from the initialized GTK main thread.
pub(crate) fn save_accel_map(path: &Path) -> Result<(), String> {
    ensure_main_thread()?;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| "accelerator path contains NUL".to_owned())?;
    unsafe { gtk::ffi::gtk_accel_map_save(path.as_ptr()) };
    Ok(())
}

/// Adds an accelerator path to GTK's process-global map.
///
/// GTK copies the path into the map during the call. The temporary `CString`
/// does not escape, and the call is restricted to the initialized GTK main
/// thread.
pub(crate) fn add_accel_entry(
    accel_path: &str,
    key: u32,
    modifiers: gdk::ModifierType,
) -> Result<(), String> {
    ensure_main_thread()?;
    let accel_path =
        CString::new(accel_path).map_err(|_| "accelerator map path contains NUL".to_owned())?;
    unsafe {
        gtk::ffi::gtk_accel_map_add_entry(accel_path.as_ptr(), key, modifiers.into_glib());
    }
    Ok(())
}

/// Changes a registered entry without exposing GTK's global map pointer.
///
/// GTK reads the path during the call and does not retain the temporary
/// `CString`. The returned boolean is copied from GTK before the wrapper
/// returns.
pub(crate) fn change_accel_entry(
    accel_path: &str,
    key: u32,
    modifiers: gdk::ModifierType,
) -> Result<bool, String> {
    ensure_main_thread()?;
    let accel_path =
        CString::new(accel_path).map_err(|_| "accelerator map path contains NUL".to_owned())?;
    Ok(unsafe {
        gtk::ffi::gtk_accel_map_change_entry(
            accel_path.as_ptr(),
            key,
            modifiers.into_glib(),
            glib::ffi::GTRUE,
        ) != glib::ffi::GFALSE
    })
}

/// Reads a registered entry into owned Rust scalar values.
///
/// GTK initializes the stack-allocated key only when it reports success. The
/// wrapper copies its integer fields after that check, and no GTK pointer
/// escapes the call.
pub(crate) fn lookup_accel_entry(
    accel_path: &str,
) -> Result<Option<(u32, gdk::ModifierType)>, String> {
    ensure_main_thread()?;
    let accel_path =
        CString::new(accel_path).map_err(|_| "accelerator map path contains NUL".to_owned())?;
    let mut key = std::mem::MaybeUninit::<gtk::ffi::GtkAccelKey>::uninit();
    let found =
        unsafe { gtk::ffi::gtk_accel_map_lookup_entry(accel_path.as_ptr(), key.as_mut_ptr()) };
    if found == glib::ffi::GFALSE {
        return Ok(None);
    }
    let key = unsafe { key.assume_init() };
    Ok(Some((key.accel_key, unsafe {
        FromGlib::from_glib(key.accel_mods)
    })))
}

fn ensure_main_thread() -> Result<(), String> {
    if gtk::is_initialized_main_thread() {
        Ok(())
    } else {
        Err("GTK accelerator maps require the initialized GTK main thread".to_owned())
    }
}
