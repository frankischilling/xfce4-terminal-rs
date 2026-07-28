//! Safe, synchronous access to the libxfconf calls used by preferences.

use std::ffi::{CStr, CString, c_char};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use glib::translate::from_glib_full;

#[repr(C)]
struct XfconfChannel {
    _private: [u8; 0],
}

#[link(name = "xfconf-0")]
unsafe extern "C" {
    fn xfconf_init(error: *mut *mut glib::ffi::GError) -> glib::ffi::gboolean;
    fn xfconf_shutdown();
    fn xfconf_list_channels() -> *mut *mut c_char;
    fn xfconf_channel_get(channel_name: *const c_char) -> *mut XfconfChannel;
    fn xfconf_channel_has_property(
        channel: *mut XfconfChannel,
        property: *const c_char,
    ) -> glib::ffi::gboolean;
    fn xfconf_channel_get_string(
        channel: *mut XfconfChannel,
        property: *const c_char,
        default_value: *const c_char,
    ) -> *mut c_char;
    fn xfconf_channel_set_string(
        channel: *mut XfconfChannel,
        property: *const c_char,
        value: *const c_char,
    ) -> glib::ffi::gboolean;
    fn xfconf_channel_get_uint(
        channel: *mut XfconfChannel,
        property: *const c_char,
        default_value: u32,
    ) -> u32;
    fn xfconf_channel_set_uint(
        channel: *mut XfconfChannel,
        property: *const c_char,
        value: u32,
    ) -> glib::ffi::gboolean;
    fn xfconf_channel_get_double(
        channel: *mut XfconfChannel,
        property: *const c_char,
        default_value: f64,
    ) -> f64;
    fn xfconf_channel_set_double(
        channel: *mut XfconfChannel,
        property: *const c_char,
        value: f64,
    ) -> glib::ffi::gboolean;
    fn xfconf_channel_get_bool(
        channel: *mut XfconfChannel,
        property: *const c_char,
        default_value: glib::ffi::gboolean,
    ) -> glib::ffi::gboolean;
    fn xfconf_channel_set_bool(
        channel: *mut XfconfChannel,
        property: *const c_char,
        value: glib::ffi::gboolean,
    ) -> glib::ffi::gboolean;
    fn xfconf_channel_reset_property(
        channel: *mut XfconfChannel,
        property: *const c_char,
        recursive: glib::ffi::gboolean,
    );
}

/// One balanced libxfconf initialization and its borrowed channel.
///
/// libxfconf owns the pointer returned by `xfconf_channel_get` until
/// `xfconf_shutdown`. The wrapper never exposes it and cannot cross threads.
/// String getters return newly allocated GLib memory; each result is copied
/// into Rust and released with `g_free`.
pub(crate) struct Session {
    channel: NonNull<XfconfChannel>,
    channel_existed: bool,
    _main_context_only: PhantomData<Rc<()>>,
}

impl Session {
    pub(crate) fn new(channel_name: &str) -> Result<Self, String> {
        let channel_name =
            CString::new(channel_name).map_err(|_| "channel name contains NUL".to_owned())?;
        let mut error = std::ptr::null_mut();

        if unsafe { xfconf_init(&mut error) } == glib::ffi::GFALSE {
            if error.is_null() {
                return Err("failed to initialize Xfconf".to_owned());
            }
            let error: glib::Error = unsafe { from_glib_full(error) };
            return Err(error.to_string());
        }

        let channel = NonNull::new(unsafe { xfconf_channel_get(channel_name.as_ptr()) });
        match channel {
            Some(channel) => {
                let channel_existed = channel_names()
                    .iter()
                    .any(|existing| existing == channel_name.to_bytes());
                Ok(Self {
                    channel,
                    channel_existed,
                    _main_context_only: PhantomData,
                })
            }
            None => {
                unsafe { xfconf_shutdown() };
                Err("Xfconf returned no channel".to_owned())
            }
        }
    }

    pub(crate) fn channel_existed(&self) -> bool {
        self.channel_existed
    }

    pub(crate) fn has(&self, property: &str) -> Result<bool, String> {
        let property = property_name(property)?;
        Ok(unsafe {
            xfconf_channel_has_property(self.channel.as_ptr(), property.as_ptr())
                != glib::ffi::GFALSE
        })
    }

    pub(crate) fn get_string(&self, property: &str) -> Result<String, String> {
        let property = property_name(property)?;
        let value = unsafe {
            xfconf_channel_get_string(self.channel.as_ptr(), property.as_ptr(), std::ptr::null())
        };
        if value.is_null() {
            return Err(format!("Xfconf returned no string for {property:?}"));
        }
        let result = unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned();
        unsafe { glib::ffi::g_free(value.cast()) };
        Ok(result)
    }

    pub(crate) fn set_string(&self, property: &str, value: &str) -> Result<(), String> {
        let property = property_name(property)?;
        let value = CString::new(value).map_err(|_| "preference contains NUL".to_owned())?;
        set_result(
            unsafe {
                xfconf_channel_set_string(self.channel.as_ptr(), property.as_ptr(), value.as_ptr())
            },
            property.as_c_str(),
        )
    }

    pub(crate) fn get_uint(&self, property: &str) -> Result<u32, String> {
        let property = property_name(property)?;
        Ok(unsafe { xfconf_channel_get_uint(self.channel.as_ptr(), property.as_ptr(), 0) })
    }

    pub(crate) fn set_uint(&self, property: &str, value: u32) -> Result<(), String> {
        let property = property_name(property)?;
        set_result(
            unsafe { xfconf_channel_set_uint(self.channel.as_ptr(), property.as_ptr(), value) },
            property.as_c_str(),
        )
    }

    pub(crate) fn get_double(&self, property: &str) -> Result<f64, String> {
        let property = property_name(property)?;
        Ok(unsafe { xfconf_channel_get_double(self.channel.as_ptr(), property.as_ptr(), 0.0) })
    }

    pub(crate) fn set_double(&self, property: &str, value: f64) -> Result<(), String> {
        let property = property_name(property)?;
        set_result(
            unsafe { xfconf_channel_set_double(self.channel.as_ptr(), property.as_ptr(), value) },
            property.as_c_str(),
        )
    }

    pub(crate) fn get_bool(&self, property: &str) -> Result<bool, String> {
        let property = property_name(property)?;
        Ok(unsafe {
            xfconf_channel_get_bool(self.channel.as_ptr(), property.as_ptr(), glib::ffi::GFALSE)
                != glib::ffi::GFALSE
        })
    }

    pub(crate) fn set_bool(&self, property: &str, value: bool) -> Result<(), String> {
        let property = property_name(property)?;
        set_result(
            unsafe {
                xfconf_channel_set_bool(self.channel.as_ptr(), property.as_ptr(), value.into())
            },
            property.as_c_str(),
        )
    }

    pub(crate) fn reset(&self, property: &str) -> Result<(), String> {
        let property = property_name(property)?;
        unsafe {
            xfconf_channel_reset_property(
                self.channel.as_ptr(),
                property.as_ptr(),
                glib::ffi::GFALSE,
            )
        };
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe { xfconf_shutdown() };
    }
}

fn property_name(name: &str) -> Result<CString, String> {
    CString::new(format!("/{name}")).map_err(|_| "preference name contains NUL".to_owned())
}

fn set_result(result: glib::ffi::gboolean, property: &CStr) -> Result<(), String> {
    if result == glib::ffi::GFALSE {
        Err(format!(
            "failed to write Xfconf property {}",
            property.to_string_lossy()
        ))
    } else {
        Ok(())
    }
}

fn channel_names() -> Vec<Vec<u8>> {
    let channels = unsafe { xfconf_list_channels() };
    if channels.is_null() {
        return Vec::new();
    }

    let mut names = Vec::new();
    let mut current = channels;
    while !unsafe { *current }.is_null() {
        names.push(unsafe { CStr::from_ptr(*current) }.to_bytes().to_vec());
        current = unsafe { current.add(1) };
    }
    unsafe { glib::ffi::g_strfreev(channels) };
    names
}
