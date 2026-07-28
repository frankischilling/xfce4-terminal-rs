//! Safe access to the small libxfce4util surface needed by the CLI.

#[link(name = "xfce4util")]
unsafe extern "C" {
    fn xfce_version_string() -> *const std::ffi::c_char;
    fn xfce_textdomain(
        package: *const std::ffi::c_char,
        locale_dir: *const std::ffi::c_char,
        encoding: *const std::ffi::c_char,
    );
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

/// Configures gettext through libxfce4util at process startup.
///
/// `xfce_textdomain` copies or internally registers all three strings before
/// returning. It changes process-global localization state, so callers must
/// invoke this once during startup before worker threads are created.
pub(crate) fn textdomain(package: &str, locale_dir: &str, encoding: &str) -> Result<(), String> {
    let package = std::ffi::CString::new(package).map_err(|_| "gettext domain contains NUL")?;
    let locale_dir = std::ffi::CString::new(locale_dir).map_err(|_| "locale path contains NUL")?;
    let encoding = std::ffi::CString::new(encoding).map_err(|_| "gettext encoding contains NUL")?;
    unsafe { xfce_textdomain(package.as_ptr(), locale_dir.as_ptr(), encoding.as_ptr()) };
    Ok(())
}
