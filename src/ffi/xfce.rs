//! Safe access to the small libxfce4util surface needed by the CLI.

#[link(name = "xfce4util")]
unsafe extern "C" {
    fn xfce_version_string() -> *const std::ffi::c_char;
}

/// Returns a copy of the version string owned by libxfce4util.
///
/// `xfce_version_string`, available in the minimum supported Xfce 4.18,
/// returns immutable process-lifetime storage and may be called from any
/// thread. The wrapper copies that borrowed string before returning, so no
/// native pointer escapes. A null pointer becomes `"unknown"`, and invalid
/// UTF-8 is replaced lossily.
pub(crate) fn version() -> String {
    let version = unsafe { xfce_version_string() };
    if version.is_null() {
        return "unknown".to_owned();
    }
    unsafe { std::ffi::CStr::from_ptr(version) }
        .to_string_lossy()
        .into_owned()
}
