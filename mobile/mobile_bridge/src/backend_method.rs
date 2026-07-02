// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki::backend::Backend;
use anki_proto_gen::descriptors_path;
use anki_proto_gen::get_services;
use prost::Message;
use prost_reflect::DescriptorPool;

pub(crate) fn backend_method(service: &str, method: &str) -> (u32, u32) {
    let pool = DescriptorPool::decode(std::fs::read(descriptors_path()).expect("descriptor pool").as_slice())
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

pub(crate) fn invoke(
    backend: &Backend,
    service: &str,
    method: &str,
    input: &[u8],
) -> Result<Vec<u8>, Vec<u8>> {
    let (service_idx, method_idx) = backend_method(service, method);
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
