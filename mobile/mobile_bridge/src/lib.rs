// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! C ABI for mobile clients. Mirrors pylib/rsbridge: protobuf bytes in/out via
//! `Backend::run_service_method`.

use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_uint;
use std::panic;
use std::ptr;
use std::slice;
use std::sync::Mutex;

use anki::backend::init_backend;
use anki::backend::Backend;

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

/// Success.
pub const ANKI_MOBILE_OK: c_int = 0;
/// Backend returned a protobuf-encoded BackendError in the output buffer.
pub const ANKI_MOBILE_BACKEND_ERROR: c_int = 1;
/// Invalid arguments or UTF-8.
pub const ANKI_MOBILE_INVALID_INPUT: c_int = 2;
/// Rust panic while handling the request.
pub const ANKI_MOBILE_PANIC: c_int = 3;

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
            let mut output = output;
            *out_len = output.len();
            let ptr = output.as_mut_ptr();
            std::mem::forget(output);
            *out_bytes = ptr;
            ANKI_MOBILE_OK
        }
        Ok(Err(err_bytes)) => {
            let mut err_bytes = err_bytes;
            *out_len = err_bytes.len();
            let ptr = err_bytes.as_mut_ptr();
            std::mem::forget(err_bytes);
            *out_bytes = ptr;
            ANKI_MOBILE_BACKEND_ERROR
        }
        Err(_) => {
            set_panic_message("panic in anki_mobile_backend_command".into());
            ANKI_MOBILE_PANIC
        }
    }
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
            let c = std::ffi::CString::new(msg).unwrap_or_default();
            *out = c.into_raw();
            ANKI_MOBILE_PANIC
        }
        None => {
            *out = ptr::null();
            ANKI_MOBILE_OK
        }
    }
}

#[cfg(test)]
mod test {
    use anki::backend::Backend;
    use anki::prelude::I18n;
    use anki_proto::backend::BackendInit;
    use anki_proto::brainlift::DashboardState;
    use anki_proto::brainlift::GetDashboardRequest;
    use anki_proto::brainlift::GetScoresResponse;
    use anki_proto::collection::OpenCollectionRequest;
    use anki_proto::generic::Empty;
    use anki_proto_gen::descriptors_path;
    use anki_proto_gen::get_services;
    use prost::Message;
    use prost_reflect::DescriptorPool;

    use super::*;

    fn backend_method(service: &str, method: &str) -> (u32, u32) {
        let pool = DescriptorPool::decode(std::fs::read(descriptors_path()).unwrap().as_slice())
            .expect("descriptor pool");
        let (_, backend) = get_services(&pool);
        let svc = backend
            .iter()
            .find(|s| s.name == service)
            .unwrap_or_else(|| panic!("missing service {service}"));
        let m = svc
            .all_methods()
            .find(|m| m.name == method)
            .unwrap_or_else(|| panic!("missing method {service}.{method}"));
        (svc.index as u32, m.index as u32)
    }

    unsafe fn backend_from_init() -> *mut AnkiMobileBackend {
        let init = BackendInit {
            preferred_langs: vec!["en".into()],
            server: false,
            locale_folder_path: String::new(),
        };
        let bytes = init.encode_to_vec();
        let mut handle: *mut AnkiMobileBackend = ptr::null_mut();
        let code = anki_mobile_backend_create(bytes.as_ptr(), bytes.len(), &mut handle);
        assert_eq!(code, ANKI_MOBILE_OK);
        handle
    }

    unsafe fn run_backend_command(
        backend: *mut AnkiMobileBackend,
        service: u32,
        method: u32,
        input: &[u8],
    ) -> Result<Vec<u8>, Vec<u8>> {
        let mut out_ptr: *mut u8 = ptr::null_mut();
        let mut out_len = 0usize;
        let code = anki_mobile_backend_command(
            backend,
            service,
            method,
            input.as_ptr(),
            input.len(),
            &mut out_ptr,
            &mut out_len,
        );
        let out = Vec::from_raw_parts(out_ptr, out_len, out_len);
        match code {
            ANKI_MOBILE_OK => Ok(out),
            ANKI_MOBILE_BACKEND_ERROR => Err(out),
            other => panic!("unexpected status code {other}"),
        }
    }

    fn normalize_scores(mut response: GetScoresResponse) -> GetScoresResponse {
        if let Some(readiness) = response.readiness.as_mut() {
            readiness.last_updated_millis = 0;
        }
        response
    }

    fn normalize_dashboard(mut response: DashboardState) -> DashboardState {
        response.computed_at_millis = 0;
        if let Some(readiness) = response.readiness.as_mut() {
            readiness.last_updated_millis = 0;
        }
        response
    }

    #[test]
    fn mobile_bridge_matches_backend_get_scores() {
        unsafe {
            let dir = std::env::temp_dir().join(format!("anki-mobile-parity-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            let mobile_path = dir.join("mobile.anki2");
            let direct_path = dir.join("direct.anki2");
            let _ = std::fs::remove_file(&mobile_path);
            let _ = std::fs::remove_file(&direct_path);

            let backend = backend_from_init();
            let open = OpenCollectionRequest {
                collection_path: mobile_path.to_string_lossy().into(),
                media_folder_path: mobile_path.with_extension("media").to_string_lossy().into(),
                media_db_path: mobile_path.with_extension("mdb").to_string_lossy().into(),
            };
            let open_bytes = open.encode_to_vec();
            let (open_service, open_method) =
                backend_method("BackendCollectionService", "open_collection");
            run_backend_command(backend, open_service, open_method, &open_bytes)
                .expect("open collection");

            let empty = Empty::default().encode_to_vec();
            let (scores_service, scores_method) =
                backend_method("BackendBrainLiftService", "get_scores");
            let mobile_out =
                run_backend_command(backend, scores_service, scores_method, &empty)
                    .expect("get_scores");

            let direct = Backend::new(I18n::template_only(), false);
            let direct_open = OpenCollectionRequest {
                collection_path: direct_path.to_string_lossy().into(),
                media_folder_path: direct_path.with_extension("media").to_string_lossy().into(),
                media_db_path: direct_path.with_extension("mdb").to_string_lossy().into(),
            };
            direct
                .run_service_method(open_service, open_method, &direct_open.encode_to_vec())
                .expect("direct open");
            let direct_out = direct
                .run_service_method(scores_service, scores_method, &empty)
                .expect("direct get_scores");
            let mobile_resp =
                GetScoresResponse::decode(mobile_out.as_slice()).expect("decode mobile");
            let direct_resp =
                GetScoresResponse::decode(direct_out.as_slice()).expect("decode direct");
            assert_eq!(normalize_scores(mobile_resp), normalize_scores(direct_resp));

            anki_mobile_backend_destroy(backend);
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn mobile_bridge_get_dashboard_matches_direct_backend() {
        unsafe {
            let dir = std::env::temp_dir().join(format!("anki-mobile-dashboard-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            let mobile_path = dir.join("mobile.anki2");
            let direct_path = dir.join("direct.anki2");
            let _ = std::fs::remove_file(&mobile_path);
            let _ = std::fs::remove_file(&direct_path);

            let backend = backend_from_init();
            let open = OpenCollectionRequest {
                collection_path: mobile_path.to_string_lossy().into(),
                media_folder_path: mobile_path.with_extension("media").to_string_lossy().into(),
                media_db_path: mobile_path.with_extension("mdb").to_string_lossy().into(),
            };
            let open_bytes = open.encode_to_vec();
            let (open_service, open_method) =
                backend_method("BackendCollectionService", "open_collection");
            run_backend_command(backend, open_service, open_method, &open_bytes)
                .expect("open collection");

            let req = GetDashboardRequest {
                recent_activity_limit: 5,
                topic_insight_limit: 3,
            }
            .encode_to_vec();
            let (dash_service, dash_method) =
                backend_method("BackendBrainLiftService", "get_dashboard");
            let mobile_out =
                run_backend_command(backend, dash_service, dash_method, &req).expect("get_dashboard");

            let direct = Backend::new(I18n::template_only(), false);
            let direct_open = OpenCollectionRequest {
                collection_path: direct_path.to_string_lossy().into(),
                media_folder_path: direct_path.with_extension("media").to_string_lossy().into(),
                media_db_path: direct_path.with_extension("mdb").to_string_lossy().into(),
            };
            direct
                .run_service_method(open_service, open_method, &direct_open.encode_to_vec())
                .expect("direct open");
            let direct_out = direct
                .run_service_method(dash_service, dash_method, &req)
                .expect("direct dashboard");
            let mobile_resp = DashboardState::decode(mobile_out.as_slice()).expect("decode mobile");
            let direct_resp = DashboardState::decode(direct_out.as_slice()).expect("decode direct");
            assert_eq!(normalize_dashboard(mobile_resp), normalize_dashboard(direct_resp));

            anki_mobile_backend_destroy(backend);
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}
