// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki::backend::Backend;
use prost::Message;

include!(concat!(env!("OUT_DIR"), "/backend_methods.rs"));

pub(crate) fn backend_method(service: &str, method: &str) -> Result<(u32, u32), Vec<u8>> {
    generated_backend_method(service, method)
        .ok_or_else(|| format!("missing backend method {service}.{method}").into_bytes())
}

pub(crate) fn invoke(
    backend: &Backend,
    service: &str,
    method: &str,
    input: &[u8],
) -> Result<Vec<u8>, Vec<u8>> {
    let (service_idx, method_idx) = backend_method(service, method)?;
    backend.run_service_method(service_idx, method_idx, input)
}

pub(crate) fn invoke_proto<M: Message + Default>(
    backend: &Backend,
    service: &str,
    method: &str,
    input: &[u8],
) -> Result<M, Vec<u8>> {
    let bytes = invoke(backend, service, method, input)?;
    M::decode(bytes.as_slice()).map_err(|err| err.to_string().into_bytes())
}
