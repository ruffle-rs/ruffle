use quick_xml::errors::Result as XmlResult;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use url::Url;

#[derive(Debug, PartialEq)]
enum PermittedPolicies {
    /// `none`
    None,
    /// `master-only`
    MasterOnly,
    /// `by-content-type`
    ByContentType,
    /// `by-ftp-filename`
    ByFtpFileName,
    /// `all`
    All,
}

#[derive(Debug)]
enum Port {
    Wildcard,
    Single(u16),
    Range(u16, u16),
}

#[derive(Debug)]
struct Access {
    domain: String,
    ports: Vec<Port>,
    secure: Option<bool>,
}

#[derive(Debug)]
struct RequestHeaders {
    domain: String,
    headers: Vec<String>,
    secure: Option<bool>,
}

#[derive(Debug)]
pub struct Policy {
    permitted: Option<PermittedPolicies>,
    allow_access_from: Vec<Access>,
    allow_request_headers_from: Vec<RequestHeaders>,
}

fn domain_and_secure_matches_url(domain: &str, secure: Option<bool>, url: &Url) -> bool {
    // XXX I think a missing `secure` attribute means `secure="true"`?
    if url.scheme() == "http" && secure.is_none_or(|secure| secure) {
        return false;
    }

    if domain == "*" {
        return true;
    }

    let Some(host) = url.host_str() else {
        return false;
    };

    if host == domain {
        return true;
    }

    // Allowing matching for `*.example.com`, but not `*example.com`.
    if let Some(wildcard) = domain.strip_prefix("*.") {
        let mut suffix = String::new();
        suffix.push('.');
        suffix.push_str(wildcard);
        return host.ends_with(&suffix);
    }

    return false;
}

impl Access {
    fn matches_url(&self, url: &Url) -> bool {
        domain_and_secure_matches_url(&self.domain, self.secure, url)
    }

    fn is_port_allowed(&self, port: u16) -> bool {
        self.ports.iter().any(|p| match *p {
            Port::Wildcard => true,
            Port::Single(v) => v == port,
            Port::Range(from, to) => port >= from && port <= to,
        })
    }
}

impl RequestHeaders {
    fn matches_url(&self, url: &Url) -> bool {
        domain_and_secure_matches_url(&self.domain, self.secure, url)
    }

    fn is_header_allowed(&self, header: &str) -> bool {
        self.headers.iter().any(|h| h == "*" || h == header)
    }
}

impl Policy {
    pub fn is_allowed(&self, url: &Url) -> bool {
        if self.permitted == Some(PermittedPolicies::None) {
            return false;
        }

        self.allow_access_from
            .iter()
            .any(|access| access.matches_url(url))
    }

    pub fn is_port_allowed(&self, url: &Url, port: u16) -> bool {
        if self.permitted == Some(PermittedPolicies::None) {
            return false;
        }

        let Some(access) = self.allow_access_from.iter().find(|h| h.matches_url(url)) else {
            return false;
        };

        access.is_port_allowed(port)
    }

    pub fn is_header_allowed(&self, url: &Url, header: &str) -> bool {
        if self.permitted == Some(PermittedPolicies::None) {
            return false;
        }

        let Some(request_headers) = self
            .allow_request_headers_from
            .iter()
            .find(|h| h.matches_url(url))
        else {
            return false;
        };

        request_headers.is_header_allowed(header)
    }
}

fn parse_site_control(e: &BytesStart) -> Option<PermittedPolicies> {
    let attr = e
        .attributes()
        .filter_map(|res| res.ok())
        .find(|attr| attr.key.into_inner() == b"permitted-cross-domain-policies")?;

    Some(match attr.value.as_ref() {
        b"none" => PermittedPolicies::None,
        b"master-only" => PermittedPolicies::MasterOnly,
        b"by-content-type" => PermittedPolicies::ByContentType,
        b"by-ftp-filename" => PermittedPolicies::ByFtpFileName,
        b"all" => PermittedPolicies::All,
        _ => return None,
    })
}

fn parse_allow_access_from(e: &BytesStart) -> XmlResult<Option<Access>> {
    let mut domain = None;
    let mut ports = Vec::new();
    let mut secure = None;
    for attr in e.attributes().filter_map(|res| res.ok()) {
        match attr.key.into_inner() {
            b"domain" => {
                domain = Some(attr.unescape_value()?.into_owned());
            }
            b"to-ports" => {
                let value = attr.unescape_value()?;

                if value == "*" {
                    ports.push(Port::Wildcard);
                    continue;
                }

                for port in value.split(',') {
                    let mut split = port.splitn(2, '-');
                    let Some(from) = split.next().and_then(|v| v.parse::<u16>().ok()) else {
                        print!("Failed to parse port: {port}");
                        continue;
                    };

                    ports.push(if let Some(v) = split.next() {
                        if let Ok(to) = v.parse() {
                            Port::Range(from, to)
                        } else {
                            println!("Failed to parse port-range: {port}");
                            continue;
                        }
                    } else {
                        Port::Single(from)
                    });
                }
            }
            b"secure" => secure = attr.as_bool(),
            _ => {
                println!("Unhandled <allow-access-from> attribute: {attr:?}");
            }
        }
    }

    Ok(domain.map(|domain| Access {
        domain,
        ports,
        secure,
    }))
}

fn parse_allow_http_request_headers_from(e: &BytesStart) -> XmlResult<Option<RequestHeaders>> {
    let mut domain = None;
    let mut headers = Vec::new();
    let mut secure = None;
    for attr in e.attributes().filter_map(|res| res.ok()) {
        match attr.key.into_inner() {
            b"domain" => {
                domain = Some(attr.unescape_value()?.into_owned());
            }
            b"headers" => {
                for header in attr.unescape_value()?.split(',') {
                    headers.push(header.into());
                }
            }
            b"secure" => secure = attr.as_bool(),
            _ => {
                println!("Unhandled <allow-http-request-headers-from> attribute: {attr:?}");
            }
        }
    }

    Ok(domain.map(|domain| RequestHeaders {
        domain,
        headers,
        secure,
    }))
}

impl Policy {
    pub fn parse(xml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut policy = Self {
            permitted: None,
            allow_access_from: vec![],
            allow_request_headers_from: vec![],
        };

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        loop {
            match reader.read_event()? {
                Event::Eof => break,
                Event::Decl(_) | Event::DocType(_) | Event::Comment(_) => continue,
                Event::Start(e) => match e.name().as_ref() {
                    b"cross-domain-policy" => loop {
                        match reader.read_event()? {
                            Event::Comment(_) => {}
                            Event::Empty(e) => match e.name().as_ref() {
                                b"site-control" => {
                                    policy.permitted = parse_site_control(&e);
                                }
                                b"allow-access-from" => {
                                    if let Some(access) = parse_allow_access_from(&e)? {
                                        policy.allow_access_from.push(access);
                                    }
                                }
                                b"allow-http-request-headers-from" => {
                                    if let Some(request_headers) =
                                        parse_allow_http_request_headers_from(&e)?
                                    {
                                        policy.allow_request_headers_from.push(request_headers);
                                    }
                                }
                                _ => {}
                            },
                            Event::End(e) if e.name().as_ref() == b"cross-domain-policy" => {
                                break;
                            }
                            event => return Err(format!("Unexpected event: {event:?}").into()),
                        }
                    },
                    _ => {
                        return Err(format!(
                            "Expected root tag <cross-domain-policy> got: {:?}",
                            e.name()
                        )
                        .into());
                    }
                },
                event => return Err(format!("Unexpected event: {event:?}").into()),
            }
        }

        Ok(policy)
    }
}
