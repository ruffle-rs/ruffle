use regress::Regex;

#[derive(Debug, Clone)]
pub struct UrlRewriteRule {
    pub pattern: Regex,
    pub replacement: String,
}

#[derive(Debug, Clone)]
pub struct CompatibilityRules {
    swf_url_rewrite_rules: Vec<UrlRewriteRule>,
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
        Self {
            swf_url_rewrite_rules: vec![],
        }
    }

    /// Default rules for general SWF compatibility.
    /// Rules that are added here must, to the best of our ability:
    /// - Only affect content that cannot run anymore, such as requiring lost assets
    /// - Not allow people to easily pirate or cheat games more than they can already
    pub fn builtin_rules() -> Self {
        Self {
            swf_url_rewrite_rules: vec![UrlRewriteRule {
                pattern: Regex::new(r"//game(\d+).konggames.com").expect("Regex must compile"),
                replacement: "//kongregate.com".to_string(),
            }],
        }
    }

    pub fn rewrite_swf_url(&self, original_url: String) -> String {
        let mut url = original_url.clone();
        for rule in &self.swf_url_rewrite_rules {
            if let Some(found) = rule.pattern.find(&url) {
                url.replace_range(found.range, &rule.replacement);
            }
        }

        if original_url != url {
            tracing::info!("Rewritten SWF url from {original_url} to {url}");
        }

        url
    }
}
