use std::fmt;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: String,
    value: String
}

impl Identifier {
    // Regex patterns for namespace and value
    const NAMESPACE_PATTERN: &'static str = r"^[a-z0-9._-]+$";
    const VALUE_PATTERN: &'static str = r"^[a-z0-9._/-]+$";

    pub fn new(namespace: Option<&str>, value: &str) -> Result<Self, String> {
        // Set the default namespace if not provided
        let namespace = namespace.unwrap_or("minecraft");

        // Validate namespace and value
        let namespace_re = Regex::new(Self::NAMESPACE_PATTERN).unwrap();
        let value_re = Regex::new(Self::VALUE_PATTERN).unwrap();

        if !namespace_re.is_match(namespace) {
            return Err(format!("Invalid namespace: '{}'", namespace));
        }

        if !value_re.is_match(value) {
            return Err(format!("Invalid value: '{}'", value));
        }

        Ok(Self {
            namespace: namespace.to_string(),
            value: value.to_string(),
        })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.value)
    }
}