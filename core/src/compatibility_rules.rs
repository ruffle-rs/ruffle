use enumset::EnumSet;
use std::borrow::Cow;
use url::ParseError as UrlParseError;
use url::Url;

use crate::backend::navigator::{ErrorResponse, FetchReason};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlRewriteStage {
    /// Perform URL rewrite before sending the request.
    /// The request will be sent to a different URL.
    BeforeRequest,

    /// Perform URL rewrite after receiving the response.
    /// The response URL will be rewritten, and SWFs will see
    /// the rewritten URL.
    AfterResponse,
}

#[derive(Debug, Clone)]
pub struct UrlRewriteRule {
    pub stage: UrlRewriteStage,
    pub fetch_reasons: EnumSet<FetchReason>,
    pub host: String,
    pub replacement: String,
}

impl UrlRewriteRule {
    pub fn new(
        stage: UrlRewriteStage,
        fetch_reasons: EnumSet<FetchReason>,
        host: impl ToString,
        replacement: impl ToString,
    ) -> Self {
        Self {
            stage,
            fetch_reasons,
            host: host.to_string(),
            replacement: replacement.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlBlockRule {
    pub fetch_reasons: EnumSet<FetchReason>,
    pub host: String,
}

impl UrlBlockRule {
    pub fn new(fetch_reasons: EnumSet<FetchReason>, host: impl ToString) -> Self {
        Self {
            fetch_reasons,
            host: host.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleSet {
    name: String,
    domain_rewrite_rules: Vec<UrlRewriteRule>,
    domain_block_rules: Vec<UrlBlockRule>,
}

#[derive(Debug, Clone)]
pub struct CompatibilityRules {
    rule_sets: Vec<RuleSet>,
}

impl Default for CompatibilityRules {
    #[cfg(feature = "default_compatibility_rules")]
    fn default() -> Self {
        Self::builtin_rules()
    }

    #[cfg(not(feature = "default_compatibility_rules"))]
    fn default() -> Self {
        Self::empty()
    }
}

impl CompatibilityRules {
    pub fn empty() -> Self {
        Self { rule_sets: vec![] }
    }

    /// Default rules for general SWF compatibility.
    /// Rules that are added here must, to the best of our ability:
    /// - Only affect content that cannot run anymore, such as requiring lost assets
    /// - Not allow people to easily pirate or cheat games more than they can already
    pub fn builtin_rules() -> Self {
        Self {
            rule_sets: vec![
                // Replaces konggames.com domains with kongregate.com to fool old sitelocks that no longer work.
                RuleSet {
                    name: "kongregate_sitelock".to_string(),
                    domain_rewrite_rules: vec![UrlRewriteRule::new(
                        UrlRewriteStage::AfterResponse,
                        EnumSet::only(FetchReason::LoadSwf),
                        "*.konggames.com",
                        "chat.kongregate.com",
                    )],
                    domain_block_rules: vec![],
                },
                // Replaces fpdownload.adobe.com with Ruffle's CDN. fpdownload.adobe.com hosts SWZ files
                // which do not work on web due to CORS (and the reliability of fpdownload.adobe.com is
                // questionable).
                RuleSet {
                    name: "fpdownload".to_string(),
                    domain_rewrite_rules: vec![UrlRewriteRule::new(
                        UrlRewriteStage::BeforeRequest,
                        EnumSet::only(FetchReason::UrlLoader),
                        "fpdownload.adobe.com",
                        "cdn.ruffle.rs",
                    )],
                    domain_block_rules: vec![],
                },
                // Mochiads currently don't work and the moachiads.com domain is up for sale.
                // There are real concerns that a malicious party could buy it.
                RuleSet {
                    name: "mochiads".to_string(),
                    domain_rewrite_rules: vec![],
                    domain_block_rules: vec![UrlBlockRule::new(EnumSet::all(), "*.mochiads.com")],
                },
            ],
        }
    }

    pub fn block_or_rewrite_swf_url(
        &self,
        original_url: Cow<'_, str>,
        stage: UrlRewriteStage,
        fetch_reason: FetchReason,
    ) -> Result<Option<String>, ErrorResponse> {
        let mut url = match Url::parse(&original_url) {
            Ok(url) => url,
            Err(UrlParseError::RelativeUrlWithoutBase) => {
                // This is an "expected" error that happens when a SWF provides
                // a relative path instead of an absolute URL. Avoid logging a
                // warning when we hit this case.
                return Ok(None);
            }
            Err(e) => {
                tracing::warn!("Couldn't rewrite swf url {original_url}: {e}");
                return Ok(None);
            }
        };
        let mut rewritten = false;

        for rule_set in &self.rule_sets {
            for rule in &rule_set.domain_rewrite_rules {
                if rule.stage != stage || !rule.fetch_reasons.contains(fetch_reason) {
                    continue;
                }

                if let Some(host) = url.host_str() {
                    if domain_matches(&rule.host, host) {
                        tracing::info!(
                            "Rewriting swf url due to compatibility ruleset '{}'",
                            rule_set.name
                        );
                        if let Err(e) = url.set_host(Some(&rule.replacement)) {
                            tracing::warn!(
                                "Couldn't rewrite swf host to {}: {e}",
                                rule.replacement
                            );
                        } else {
                            rewritten = true;
                        }
                    }
                }
            }

            if stage == UrlRewriteStage::BeforeRequest {
                for rule in &rule_set.domain_block_rules {
                    if !rule.fetch_reasons.contains(fetch_reason) {
                        continue;
                    }

                    if let Some(host) = url.host_str() {
                        if domain_matches(&rule.host, host) {
                            tracing::info!(
                                "Blocking url due to compatibility ruleset '{}'",
                                rule_set.name
                            );

                            return Err(ErrorResponse {
                                url: original_url.to_string(),
                                error: crate::loader::Error::BlockedHost(rule.host.clone()),
                            });
                        }
                    }
                }
            }
        }

        if rewritten {
            Ok(Some(url.to_string()))
        } else {
            Ok(None)
        }
    }
}

/// Tests that two domains match.
///
/// Expected string may start with `*.` to allow for any further subdomains.
pub fn domain_matches(expected: &str, actual: &str) -> bool {
    let mut expected_parts = expected.rsplit('.').peekable();
    let mut actual_parts = actual.rsplit('.');
    let mut allow_subdomains = false;

    while let Some(test) = expected_parts.next() {
        if test == "*" && expected_parts.peek().is_none() {
            allow_subdomains = true;
            continue;
        }

        match actual_parts.next() {
            Some(actual) => {
                if !test.eq_ignore_ascii_case(actual) {
                    return false;
                }
            }
            None => return false,
        }
    }

    allow_subdomains || actual_parts.next().is_none()
}

#[cfg(test)]
mod tests {
    use crate::compatibility_rules::domain_matches;

    #[test]
    fn test_domain_matches() {
        assert!(domain_matches("foo.example.com", "foo.example.com"));
        assert!(domain_matches("*.example.com", "example.com"));
        assert!(domain_matches("*.example.com", "foo.example.com"));
        assert!(domain_matches("*.foo.example.com", "foo.example.com"));
        assert!(domain_matches("*.com", "foo.example.com"));
        assert!(domain_matches("*", "foo.example.com"));
        assert!(!domain_matches("", "foo.example.com"));
        assert!(!domain_matches("com", "foo.example.com"));
        assert!(!domain_matches("example.com", "foo.example.com"));
        assert!(!domain_matches("bar.example.com", "foo.example.com"));
        assert!(!domain_matches("bar.foo.example.com", "foo.example.com"));
    }
}
