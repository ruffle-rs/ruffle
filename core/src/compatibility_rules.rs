use url::Url;

#[derive(Debug, Clone)]
pub struct UrlRewriteRule {
    pub host: String,
    pub replacement: String,
}

impl UrlRewriteRule {
    pub fn new(host: impl ToString, replacement: impl ToString) -> Self {
        Self {
            host: host.to_string(),
            replacement: replacement.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleSet {
    name: String,
    swf_domain_rewrite_rules: Vec<UrlRewriteRule>,
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
        // Replaces konggames.com domains with kongregate.com to fool old sitelocks that no longer work.
        let kongregate_sitelock = RuleSet {
            name: "kongregate_sitelock".to_string(),
            swf_domain_rewrite_rules: vec![UrlRewriteRule::new(
                "*.konggames.com",
                "chat.kongregate.com",
            )],
        };

        Self {
            rule_sets: vec![kongregate_sitelock],
        }
    }

    pub fn rewrite_swf_url(&self, original_url: String) -> String {
        let mut url = match Url::parse(&original_url) {
            Ok(url) => url,
            Err(e) => {
                tracing::warn!("Couldn't rewrite swf url {original_url}: {e}");
                return original_url;
            }
        };

        for rule_set in &self.rule_sets {
            for rule in &rule_set.swf_domain_rewrite_rules {
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
                        }
                    }
                }
            }
        }

        url.to_string()
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
