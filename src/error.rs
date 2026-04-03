use core::fmt;

/// Error returned when a version or range string cannot be parsed.
///
/// # Examples
///
/// ```rust
/// use js_semver::{SemverError, Version};
///
/// let err: SemverError = Version::parse("1.a.b").unwrap_err();
/// eprintln!("{err}");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemverError(String);

impl fmt::Display for SemverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SemverError {}

impl SemverError {
    pub(crate) fn new(s: impl fmt::Display) -> Self {
        Self(s.to_string())
    }
}
