use core::fmt;

/// A structured semver parse error classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemverErrorKind {
    /// A non-semver suffix or other unexpected character was found.
    UnexpectedCharacter(char),
    /// The input exceeded the maximum accepted length.
    MaxLengthExceeded,
    /// The input exceeded `MAX_SAFE_INTEGER`.
    MaxSafeIntegerExceeded,
    /// The entire input was empty.
    Empty,
    /// An empty segment was encountered.
    EmptySegment,
    /// A partial version ended with a dot.
    TrailingDot,
    /// A dot appeared in an unexpected position.
    UnexpectedDot,
    /// A numeric component had a leading zero.
    LeadingZero,
    /// A numeric component was invalid.
    InvalidNumber,
    /// A required version component was missing.
    MissingVersionSegment,
    /// An operator was not followed by a version.
    MissingVersionAfterOperator(&'static str),
}

impl fmt::Display for SemverErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter(ch) => write!(f, "unexpected character: '{ch}'"),
            Self::MaxLengthExceeded => f.write_str("maximum length of 256 characters exceeded"),
            Self::MaxSafeIntegerExceeded => f.write_str("number exceeds MAX_SAFE_INTEGER"),
            Self::Empty => f.write_str("empty"),
            Self::EmptySegment => f.write_str("empty segment"),
            Self::TrailingDot => f.write_str("trailing dot"),
            Self::UnexpectedDot => f.write_str("unexpected dot"),
            Self::LeadingZero => f.write_str("leading zero"),
            Self::InvalidNumber => f.write_str("invalid number"),
            Self::MissingVersionSegment => f.write_str("missing version segment"),
            Self::MissingVersionAfterOperator(operator) => {
                write!(f, "missing version after {operator}")
            }
        }
    }
}

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
///
/// # Note
///
/// Do not depend on exact error message strings. The `Display` output is
/// intended for humans and may change between releases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemverError {
    kind: SemverErrorKind,
}

impl fmt::Display for SemverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SemverError {}

impl From<SemverErrorKind> for SemverError {
    fn from(kind: SemverErrorKind) -> Self {
        Self { kind }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    use super::{SemverError, SemverErrorKind};

    #[test]
    fn semver_error_kind_display_variants() {
        let cases = [
            (
                SemverErrorKind::UnexpectedCharacter('x'),
                "unexpected character: 'x'",
            ),
            (
                SemverErrorKind::MaxLengthExceeded,
                "maximum length of 256 characters exceeded",
            ),
            (
                SemverErrorKind::MaxSafeIntegerExceeded,
                "number exceeds MAX_SAFE_INTEGER",
            ),
            (SemverErrorKind::Empty, "empty"),
            (SemverErrorKind::EmptySegment, "empty segment"),
            (SemverErrorKind::TrailingDot, "trailing dot"),
            (SemverErrorKind::UnexpectedDot, "unexpected dot"),
            (SemverErrorKind::LeadingZero, "leading zero"),
            (SemverErrorKind::InvalidNumber, "invalid number"),
            (
                SemverErrorKind::MissingVersionSegment,
                "missing version segment",
            ),
            (
                SemverErrorKind::MissingVersionAfterOperator(">="),
                "missing version after >=",
            ),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.to_string(), expected);
            let error: SemverError = kind.into();
            assert_eq!(error.to_string(), expected);
        }
    }
}
