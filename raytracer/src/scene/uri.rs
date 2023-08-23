
use uriparse::*;

pub enum UriResolved<'a> {
    Buffer,
    Base64(&'a str)
}

pub fn resolve_uri<'a>(uri_input: &'a str) -> anyhow::Result<UriResolved> {
    let uri = uri::URI::try_from(uri_input).expect(format!("Uri is corrupted {}", uri_input).as_str());
    let scheme = uri.scheme();
    match scheme.as_str() {
        "data" => {
            match uri_input.find(";base64,") {
                None => {
                    panic!("resolve_uri: data: protocol but not a base64 uri");
                }
                Some(index) => {
                    let slice = &uri_input[index..];
                    return Ok(UriResolved::Base64(slice));
                }
            }
        }
        _ => {
            panic!("unsupported protocol");
        }
    }
}