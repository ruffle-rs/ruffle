//! The `multipart/form-data` body that `FileReference.upload` sends.

use indexmap::IndexMap;

/// A `multipart/form-data` request body, and the boundary that separates
/// its parts.
pub struct MultipartBody {
    pub boundary: String,
    pub body: Vec<u8>,
}

impl MultipartBody {
    /// The `Content-Type` to send this body with.
    pub fn content_type(&self) -> String {
        format!("multipart/form-data; boundary={}", self.boundary)
    }
}

/// Builds the body Flash Player sends for `FileReference.upload`: a
/// `Filename` field, then the request's variables, then the file in a field
/// named `data_field`, then an `Upload` field with the value `Submit Query`.
///
/// Flash Player picks a random boundary of 10 dashes and 30 letters and
/// digits. This picks one of the same shape, in lowercase, that does not
/// occur in any part, but deterministically, so that the same upload gives
/// the same body.
pub fn upload_body(
    file_name: &str,
    fields: &IndexMap<String, String>,
    data_field: &str,
    data: &[u8],
) -> MultipartBody {
    let mut seed = 0;
    loop {
        let boundary = boundary(seed);
        let body = upload_body_with_boundary(&boundary, file_name, fields, data_field, data);
        // The body contains the boundary once per part, plus the closing one.
        let parts = fields.len() + 3;
        if count_occurrences(&body, boundary.as_bytes()) == parts + 1 {
            return MultipartBody { boundary, body };
        }
        seed += 1;
    }
}

fn upload_body_with_boundary(
    boundary: &str,
    file_name: &str,
    fields: &IndexMap<String, String>,
    data_field: &str,
    data: &[u8],
) -> Vec<u8> {
    let mut body = Vec::with_capacity(data.len() + 512);
    let field = |body: &mut Vec<u8>, name: &str, value: &[u8]| {
        part_header(body, boundary, name, None);
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(value);
        body.extend_from_slice(b"\r\n");
    };

    field(&mut body, "Filename", file_name.as_bytes());
    for (name, value) in fields {
        field(&mut body, name, value.as_bytes());
    }

    part_header(&mut body, boundary, data_field, Some(file_name));
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(data);
    body.extend_from_slice(b"\r\n");

    field(&mut body, "Upload", b"Submit Query");

    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");
    body
}

/// Writes a part's boundary line and `Content-Disposition` header, with a
/// `filename` for a file, ending with the header's CRLF.
fn part_header(body: &mut Vec<u8>, boundary: &str, name: &str, file_name: Option<&str>) {
    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"\r\nContent-Disposition: form-data; name=\"");
    body.extend_from_slice(quote(name).as_bytes());
    body.extend_from_slice(b"\"");
    if let Some(file_name) = file_name {
        body.extend_from_slice(b"; filename=\"");
        body.extend_from_slice(quote(file_name).as_bytes());
        body.extend_from_slice(b"\"");
    }
    body.extend_from_slice(b"\r\n");
}

/// Escapes a name for a quoted header parameter, as browsers do (WHATWG
/// HTML's multipart/form-data encoding).
fn quote(s: &str) -> String {
    s.replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace('"', "%22")
}

/// A boundary of the shape Flash Player uses, from a seed.
fn boundary(seed: u64) -> String {
    // Flash Player mixes cases, but browsers lowercase a Blob's type, and
    // with it the boundary in the Content-Type that the web player sends.
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    // splitmix64, so each seed gives an unrelated-looking boundary.
    let mut state = seed.wrapping_add(0x5275_6666_6c65);
    let mut next = || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let mut boundary = "-".repeat(10);
    for _ in 0..30 {
        boundary.push(CHARS[(next() % CHARS.len() as u64) as usize] as char);
    }
    boundary
}

fn count_occurrences(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn body_has_flash_players_parts_in_order() {
        let vars = fields(&[("type", "background-image"), ("s", "abc")]);
        let body = upload_body_with_boundary("B", "bg.png", &vars, "Filedata", b"\x89PNG");
        let expected: &[u8] = b"--B\r\n\
            Content-Disposition: form-data; name=\"Filename\"\r\n\r\n\
            bg.png\r\n\
            --B\r\n\
            Content-Disposition: form-data; name=\"type\"\r\n\r\n\
            background-image\r\n\
            --B\r\n\
            Content-Disposition: form-data; name=\"s\"\r\n\r\n\
            abc\r\n\
            --B\r\n\
            Content-Disposition: form-data; name=\"Filedata\"; filename=\"bg.png\"\r\n\
            Content-Type: application/octet-stream\r\n\r\n\
            \x89PNG\r\n\
            --B\r\n\
            Content-Disposition: form-data; name=\"Upload\"\r\n\r\n\
            Submit Query\r\n\
            --B--\r\n";
        assert_eq!(
            String::from_utf8_lossy(&body),
            String::from_utf8_lossy(expected)
        );
    }

    #[test]
    fn data_field_name_is_used() {
        let body = upload_body_with_boundary("B", "a.txt", &fields(&[]), "upfile", b"x");
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("name=\"upfile\"; filename=\"a.txt\""));
        assert!(!text.contains("Filedata"));
    }

    #[test]
    fn quotes_in_names_are_escaped() {
        let body = upload_body_with_boundary("B", "a\"b.txt", &fields(&[("x\"y", "1")]), "f", b"");
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("name=\"x%22y\""));
        assert!(text.contains("filename=\"a%22b.txt\""));
        // The Filename field's value is the name as it is.
        assert!(text.contains("\r\n\r\na\"b.txt\r\n"));
    }

    #[test]
    fn boundary_has_flash_players_shape() {
        let upload = upload_body("a.txt", &fields(&[]), "Filedata", b"hello");
        assert_eq!(upload.boundary.len(), 40);
        assert!(upload.boundary.starts_with("----------"));
        // Lowercase, because browsers lowercase a Blob's type, and with it
        // the boundary in the Content-Type that Ruffle's web player sends.
        assert!(
            upload.boundary[10..]
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        );
        assert_eq!(
            upload.content_type(),
            format!("multipart/form-data; boundary={}", upload.boundary)
        );
    }

    #[test]
    fn boundary_is_deterministic() {
        let a = upload_body("a.txt", &fields(&[]), "Filedata", b"hello");
        let b = upload_body("a.txt", &fields(&[]), "Filedata", b"hello");
        assert_eq!(a.boundary, b.boundary);
        assert_eq!(a.body, b.body);
    }

    #[test]
    fn boundary_avoids_the_data() {
        let first = boundary(0);
        let data = format!("--{first}\r\n").into_bytes();
        let upload = upload_body("a.txt", &fields(&[]), "Filedata", &data);
        assert_ne!(upload.boundary, first);
        assert_eq!(
            count_occurrences(&upload.body, upload.boundary.as_bytes()),
            4
        );
    }
}
