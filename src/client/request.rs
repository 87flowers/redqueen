use crate::{
    client::domain::Remote,
    common::{headers, time::unix_time},
};
use axum::http::HeaderValue;
use getrandom::{
    SysRng,
    rand_core::{Rng, UnwrapErr},
};
use reqwest::{
    Client, Method, RequestBuilder,
    header::{CONTENT_LENGTH, HeaderMap},
};
use sha2::Digest;

pub struct BodyMeta {
    size: usize,
    hash: [u8; 64],
}

impl BodyMeta {
    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = sha2::Sha512::new();
        hasher.update(bytes);
        Self { size: bytes.len(), hash: hasher.finalize().into() }
    }
}

pub fn build_headers(remote: &Remote, method: Method, path: &str, body_meta: BodyMeta) -> HeaderMap {
    let mut csprng = UnwrapErr(SysRng);

    let timestamp = unix_time();
    let nonce = csprng.next_u64();
    let body_hash = const_hex::encode(body_meta.hash);

    let signature_payload = format!("RedQueen {method} {path} {} {} {timestamp} {nonce}", body_meta.size, body_hash);
    let signature = remote.private_key.sign(signature_payload.as_bytes());

    fn header_value(src: &str) -> HeaderValue {
        // SAFETY: All values of src below should not have invalid characters
        HeaderValue::from_str(src).expect("Invalid string passed to HeaderValue")
    }

    fn sensitive_value(src: &str) -> HeaderValue {
        let mut value = header_value(src);
        value.set_sensitive(true);
        value
    }

    let mut headers = HeaderMap::new();
    headers.insert(headers::RQ_TIMESTAMP, header_value(&timestamp.to_string()));
    headers.insert(headers::RQ_NONCE, header_value(&nonce.to_string()));
    headers.insert(headers::RQ_BODY_HASH, header_value(&body_hash));
    headers.insert(headers::RQ_SIGNATURE, sensitive_value(&signature.to_string()));
    headers.insert(headers::RQ_USERNAME, sensitive_value(&const_hex::encode(remote.username.as_bytes())));
    headers.insert(headers::RQ_PUBLIC_KEY, sensitive_value(&remote.public_key.to_string()));
    headers.insert(CONTENT_LENGTH, header_value(&body_meta.size.to_string()));
    headers
}

pub fn build_request(
    client: &Client, remote: &Remote, method: Method, path: &str, body_meta: BodyMeta,
) -> RequestBuilder {
    let url = remote.url.join(path).expect("Invalid remote url or path");
    client.request(method.clone(), url).headers(build_headers(remote, method, path, body_meta))
}
