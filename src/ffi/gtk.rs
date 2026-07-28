//! Safe wrappers for GTK accelerator-map persistence.

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
pub(crate) fn save_accel_map(path: &Path) -> Result<(), String> {
    ensure_main_thread()?;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| "accelerator path contains NUL".to_owned())?;
    unsafe { gtk::ffi::gtk_accel_map_save(path.as_ptr()) };
    Ok(())
}

fn ensure_main_thread() -> Result<(), String> {
    if gtk::is_initialized_main_thread() {
        Ok(())
    } else {
        Err("GTK accelerator maps require the initialized GTK main thread".to_owned())
    }
}
