// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! C ABI for mobile clients. Mirrors pylib/rsbridge: protobuf bytes in/out via
//! `Backend::run_service_method`.

// Safety contracts for the `extern "C"` entry points (pointer validity, buffer
// ownership, freeing via `anki_mobile_bytes_free`) are documented in the public
// header `include/anki_mobile.h`.
#![allow(clippy::missing_safety_doc)]

mod backend_method;
mod demo_pages;
mod gre_pages;
#[cfg(test)]
mod ios_demo_bundle;
#[cfg(test)]
mod parity;
mod study_pages;
mod sync_pages;

use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::panic;
use std::ptr;
use std::slice;
use std::sync::Mutex;

use anki::backend::init_backend;
use anki::backend::Backend;
use serde::Serialize;

struct MobileBackend {
    backend: Backend,
}

static PANIC_MESSAGE: Mutex<Option<String>> = Mutex::new(None);

fn set_panic_message(msg: String) {
    if let Ok(mut guard) = PANIC_MESSAGE.lock() {
        *guard = Some(msg);
    }
}

fn take_panic_message() -> Option<String> {
    PANIC_MESSAGE.lock().ok()?.take()
}

/// Opaque handle to an Anki backend instance.
pub struct AnkiMobileBackend(MobileBackend);

impl AnkiMobileBackend {
    #[cfg(test)]
    pub(crate) fn backend(&self) -> &Backend {
        &self.0.backend
    }
}

/// Success.
pub const ANKI_MOBILE_OK: c_int = 0;
/// Backend returned a protobuf-encoded BackendError in the output buffer.
pub const ANKI_MOBILE_BACKEND_ERROR: c_int = 1;
/// Invalid arguments or UTF-8.
pub const ANKI_MOBILE_INVALID_INPUT: c_int = 2;
/// Rust panic while handling the request.
pub const ANKI_MOBILE_PANIC: c_int = 3;

unsafe fn write_bytes(mut bytes: Vec<u8>, out_bytes: *mut *mut u8, out_len: *mut usize) {
    *out_len = bytes.len();
    let ptr = bytes.as_mut_ptr();
    std::mem::forget(bytes);
    *out_bytes = ptr;
}

unsafe fn write_json<T: Serialize>(
    value: &T,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    match serde_json::to_vec(value) {
        Ok(bytes) => {
            write_bytes(bytes, out_bytes, out_len);
            ANKI_MOBILE_OK
        }
        Err(_) => ANKI_MOBILE_INVALID_INPUT,
    }
}

unsafe fn with_backend<F>(backend: *mut AnkiMobileBackend, f: F) -> c_int
where
    F: FnOnce(&Backend) -> c_int,
{
    if backend.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| f(&(*backend).0.backend)));
    match result {
        Ok(code) => code,
        Err(_) => {
            set_panic_message("panic in mobile_bridge".into());
            ANKI_MOBILE_PANIC
        }
    }
}

#[no_mangle]
pub extern "C" fn anki_mobile_buildhash() -> *const c_char {
    anki::version::buildhash().as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_backend_create(
    init_msg: *const u8,
    init_len: usize,
    out_backend: *mut *mut AnkiMobileBackend,
) -> c_int {
    if init_msg.is_null() || out_backend.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    let init_bytes = slice::from_raw_parts(init_msg, init_len);
    match init_backend(init_bytes) {
        Ok(backend) => {
            let handle = Box::new(AnkiMobileBackend(MobileBackend { backend }));
            *out_backend = Box::into_raw(handle);
            ANKI_MOBILE_OK
        }
        Err(_) => ANKI_MOBILE_INVALID_INPUT,
    }
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_backend_destroy(backend: *mut AnkiMobileBackend) {
    if !backend.is_null() {
        drop(Box::from_raw(backend));
    }
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_open_collection(
    backend: *mut AnkiMobileBackend,
    collection_path: *const c_char,
    media_folder_path: *const c_char,
    media_db_path: *const c_char,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if collection_path.is_null()
        || media_folder_path.is_null()
        || media_db_path.is_null()
        || out_bytes.is_null()
        || out_len.is_null()
    {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(backend, |backend| {
        let collection_path = match CStr::from_ptr(collection_path).to_str() {
            Ok(value) => value,
            Err(_) => return ANKI_MOBILE_INVALID_INPUT,
        };
        let media_folder_path = match CStr::from_ptr(media_folder_path).to_str() {
            Ok(value) => value,
            Err(_) => return ANKI_MOBILE_INVALID_INPUT,
        };
        let media_db_path = match CStr::from_ptr(media_db_path).to_str() {
            Ok(value) => value,
            Err(_) => return ANKI_MOBILE_INVALID_INPUT,
        };
        match gre_pages::open_collection(backend, collection_path, media_folder_path, media_db_path)
        {
            Ok(()) => ANKI_MOBILE_OK,
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_backend_command(
    backend: *mut AnkiMobileBackend,
    service: c_uint,
    method: c_uint,
    input: *const u8,
    input_len: usize,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if backend.is_null() || input.is_null() || out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let backend = &(*backend).0.backend;
        let input_bytes = slice::from_raw_parts(input, input_len);
        backend.run_service_method(service, method, input_bytes)
    }));

    match result {
        Ok(Ok(output)) => {
            write_bytes(output, out_bytes, out_len);
            ANKI_MOBILE_OK
        }
        Ok(Err(err_bytes)) => {
            write_bytes(err_bytes, out_bytes, out_len);
            ANKI_MOBILE_BACKEND_ERROR
        }
        Err(_) => {
            set_panic_message("panic in anki_mobile_backend_command".into());
            ANKI_MOBILE_PANIC
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_dashboard_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(backend, |backend| {
        match gre_pages::load_dashboard_page(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_progress_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(backend, |backend| {
        match gre_pages::load_progress_page(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_practice_bootstrap_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(
        backend,
        |backend| match gre_pages::load_practice_bootstrap(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        },
    )
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_record_attempt_json(
    backend: *mut AnkiMobileBackend,
    input: *const u8,
    input_len: usize,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if backend.is_null() || input.is_null() || out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    let input_bytes = slice::from_raw_parts(input, input_len);
    let request: gre_pages::GreRecordAttemptInput = match serde_json::from_slice(input_bytes) {
        Ok(value) => value,
        Err(_) => return ANKI_MOBILE_INVALID_INPUT,
    };
    with_backend(
        backend,
        |backend| match gre_pages::record_practice_attempt(backend, request) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        },
    )
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_practice_scores_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(
        backend,
        |backend| match gre_pages::load_practice_score_strip(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        },
    )
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_study_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(backend, |backend| {
        match gre_pages::load_study_page(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_study_review_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(backend, |backend| {
        match study_pages::load_study_review(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_gre_study_answer_json(
    backend: *mut AnkiMobileBackend,
    input: *const u8,
    input_len: usize,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if backend.is_null() || input.is_null() || out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    let input_bytes = slice::from_raw_parts(input, input_len);
    let request: study_pages::GreStudyAnswerInput = match serde_json::from_slice(input_bytes) {
        Ok(value) => value,
        Err(_) => return ANKI_MOBILE_INVALID_INPUT,
    };
    with_backend(backend, |backend| {
        match study_pages::answer_study_card(backend, request) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_prepare_demo_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(
        backend,
        |backend| match demo_pages::prepare_demo_collection(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        },
    )
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_brainlift_sync_status_json(
    backend: *mut AnkiMobileBackend,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    with_backend(backend, |backend| {
        match sync_pages::load_sync_status(backend) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_brainlift_sync_pull_json(
    backend: *mut AnkiMobileBackend,
    input: *const u8,
    input_len: usize,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if backend.is_null() || input.is_null() || out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    let input_bytes = slice::from_raw_parts(input, input_len);
    let request: sync_pages::GreAtlasSyncPullInput = match serde_json::from_slice(input_bytes) {
        Ok(value) => value,
        Err(_) => return ANKI_MOBILE_INVALID_INPUT,
    };
    with_backend(backend, |backend| {
        match sync_pages::pull_sync_changes(backend, request) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_brainlift_sync_push_json(
    backend: *mut AnkiMobileBackend,
    input: *const u8,
    input_len: usize,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    if backend.is_null() || input.is_null() || out_bytes.is_null() || out_len.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    let input_bytes = slice::from_raw_parts(input, input_len);
    let request: sync_pages::GreAtlasSyncPushInput = match serde_json::from_slice(input_bytes) {
        Ok(value) => value,
        Err(_) => return ANKI_MOBILE_INVALID_INPUT,
    };
    with_backend(backend, |backend| {
        match sync_pages::push_sync_changes(backend, request) {
            Ok(view) => write_json(&view, out_bytes, out_len),
            Err(err_bytes) => {
                write_bytes(err_bytes, out_bytes, out_len);
                ANKI_MOBILE_BACKEND_ERROR
            }
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_bytes_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[no_mangle]
pub unsafe extern "C" fn anki_mobile_last_error(out: *mut *const c_char) -> c_int {
    if out.is_null() {
        return ANKI_MOBILE_INVALID_INPUT;
    }
    match take_panic_message() {
        Some(msg) => {
            let c = CString::new(msg).unwrap_or_default();
            *out = c.into_raw();
            ANKI_MOBILE_PANIC
        }
        None => {
            *out = ptr::null();
            ANKI_MOBILE_OK
        }
    }
}
