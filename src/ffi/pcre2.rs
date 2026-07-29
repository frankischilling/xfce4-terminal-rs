//! Safe wrappers for the PCRE2 calls that classify terminal links.
//!
//! The wrappers target the 8-bit PCRE2 interface of the system library, the
//! same one VTE and the C reference link against. A compiled pattern owns its
//! native code block and frees it on drop. PCRE2 keeps no global state and
//! never alters a compiled pattern while matching, so a pattern may be shared
//! between threads; each match allocates and releases its own match data
//! instead. Subjects are borrowed for the duration of a call, and no native
//! pointer escapes these functions.

use std::ffi::{c_int, c_void};

/// PCRE2's report that a subject does not match.
const ERROR_NOMATCH: c_int = -1;

#[repr(C)]
struct Code {
    _opaque: [u8; 0],
}

#[repr(C)]
struct MatchData {
    _opaque: [u8; 0],
}

#[link(name = "pcre2-8")]
unsafe extern "C" {
    fn pcre2_compile_8(
        pattern: *const u8,
        length: usize,
        options: u32,
        error_code: *mut c_int,
        error_offset: *mut usize,
        context: *mut c_void,
    ) -> *mut Code;
    fn pcre2_code_free_8(code: *mut Code);
    fn pcre2_match_data_create_from_pattern_8(
        code: *const Code,
        context: *mut c_void,
    ) -> *mut MatchData;
    fn pcre2_match_data_free_8(match_data: *mut MatchData);
    fn pcre2_match_8(
        code: *const Code,
        subject: *const u8,
        length: usize,
        start_offset: usize,
        options: u32,
        match_data: *mut MatchData,
        context: *mut c_void,
    ) -> c_int;
}

/// A compiled PCRE2 pattern.
pub(crate) struct Pattern {
    code: *mut Code,
}

// SAFETY: PCRE2 has no global or static state, and a compiled pattern is
// immutable once `pcre2_compile_8` returns. The library documents that such a
// pattern may be used by several threads at the same time. These wrappers
// never request just-in-time compilation, which is the one feature that would
// need per-thread stacks.
unsafe impl Send for Pattern {}
unsafe impl Sync for Pattern {}

impl Pattern {
    /// Compiles a pattern with the given options.
    ///
    /// The pattern text is borrowed for the call only, because PCRE2 copies
    /// what it needs into the returned code block. A failure yields the native
    /// error number, which is the value the C reference reports.
    pub(crate) fn compile(pattern: &str, options: u32) -> Result<Self, i32> {
        let mut error_code: c_int = 0;
        let mut error_offset: usize = 0;
        let code = unsafe {
            pcre2_compile_8(
                pattern.as_ptr(),
                pattern.len(),
                options,
                &mut error_code,
                &mut error_offset,
                std::ptr::null_mut(),
            )
        };

        if code.is_null() {
            Err(error_code)
        } else {
            Ok(Self { code })
        }
    }

    /// Reports whether the subject matches anywhere.
    ///
    /// The match data block lives only inside this call, so the borrowed
    /// subject cannot outlive the native state that refers to it. Match
    /// failures other than "no match" are returned as the native error number.
    pub(crate) fn matches(&self, subject: &str) -> Result<bool, i32> {
        let match_data =
            unsafe { pcre2_match_data_create_from_pattern_8(self.code, std::ptr::null_mut()) };
        if match_data.is_null() {
            return Err(0);
        }

        let result = unsafe {
            pcre2_match_8(
                self.code,
                subject.as_ptr(),
                subject.len(),
                0,
                0,
                match_data,
                std::ptr::null_mut(),
            )
        };
        unsafe { pcre2_match_data_free_8(match_data) };

        match result {
            0.. => Ok(true),
            ERROR_NOMATCH => Ok(false),
            error => Err(error),
        }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe { pcre2_code_free_8(self.code) };
    }
}
