use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Position {
    Major,
    Minor,
    Patch,
    PreRelease,
    BuildMetadata,
}

/// A structured semver parse error classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SemverErrorKind {
    /// An unexpected character was found while parsing a version component.
    UnexpectedCharacterWhileParsing(char, Position),
    /// An unexpected character was found after a version component.
    UnexpectedCharacterAfter(char, Position),
    /// A concrete version component followed a wildcard component.
    UnexpectedCharacterAfterWildcard,
    /// The input exceeded the maximum accepted length.
    MaxLengthExceeded,
    /// The input exceeded `MAX_SAFE_INTEGER`.
    MaxSafeIntegerExceeded(Position),
    /// The entire input was empty.
    Empty,
    /// An empty pre-release or build metadata identifier was encountered.
    EmptyIdentifierSegment(Position),
    /// A numeric component had a leading zero.
    LeadingZero(Position),
    /// A required version component was missing.
    MissingVersionSegment(Position),
    /// An operator was not followed by a version.
    MissingVersionAfterOperator(&'static str),
}

impl fmt::Display for SemverErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedCharacterWhileParsing(ch, position) => write!(
                f,
                "unexpected character '{}' while parsing {}",
                ch.escape_debug(),
                position.description()
            ),
            Self::UnexpectedCharacterAfter(ch, position) => write!(
                f,
                "unexpected character '{}' after {}",
                ch.escape_debug(),
                position.description()
            ),
            Self::UnexpectedCharacterAfterWildcard => {
                f.write_str("unexpected character after wildcard in version range")
            }
            Self::MaxLengthExceeded => f.write_str("maximum length of 256 characters exceeded"),
            Self::MaxSafeIntegerExceeded(position) => write!(
                f,
                "number exceeds MAX_SAFE_INTEGER in {}",
                position.description()
            ),
            Self::Empty => f.write_str("empty"),
            Self::EmptyIdentifierSegment(position) => {
                write!(f, "empty identifier segment in {}", position.description())
            }
            Self::LeadingZero(position) => {
                write!(f, "invalid leading zero in {}", position.description())
            }
            Self::MissingVersionSegment(position) => {
                write!(f, "missing {} version segment", position.version_name())
            }
            Self::MissingVersionAfterOperator(operator) => {
                write!(f, "missing version after {operator}")
            }
        }
    }
}

impl Position {
    fn description(self) -> &'static str {
        match self {
            Self::Major => "major version",
            Self::Minor => "minor version",
            Self::Patch => "patch version",
            Self::PreRelease => "pre-release identifier",
            Self::BuildMetadata => "build metadata",
        }
    }

    fn version_name(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::PreRelease => "pre-release",
            Self::BuildMetadata => "build metadata",
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
impl core::error::Error for SemverError {}

impl From<SemverErrorKind> for SemverError {
    fn from(kind: SemverErrorKind) -> Self {
        Self { kind }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    use super::{Position, SemverError, SemverErrorKind};

    #[test]
    fn semver_error_kind_display_variants() {
        let cases = [
            (
                SemverErrorKind::UnexpectedCharacterWhileParsing('x', Position::Minor),
                "unexpected character 'x' while parsing minor version",
            ),
            (
                SemverErrorKind::UnexpectedCharacterAfter('x', Position::Patch),
                "unexpected character 'x' after patch version",
            ),
            (
                SemverErrorKind::UnexpectedCharacterAfterWildcard,
                "unexpected character after wildcard in version range",
            ),
            (
                SemverErrorKind::MaxLengthExceeded,
                "maximum length of 256 characters exceeded",
            ),
            (
                SemverErrorKind::MaxSafeIntegerExceeded(Position::Major),
                "number exceeds MAX_SAFE_INTEGER in major version",
            ),
            (SemverErrorKind::Empty, "empty"),
            (
                SemverErrorKind::EmptyIdentifierSegment(Position::PreRelease),
                "empty identifier segment in pre-release identifier",
            ),
            (
                SemverErrorKind::EmptyIdentifierSegment(Position::BuildMetadata),
                "empty identifier segment in build metadata",
            ),
            (
                SemverErrorKind::LeadingZero(Position::PreRelease),
                "invalid leading zero in pre-release identifier",
            ),
            (
                SemverErrorKind::MissingVersionSegment(Position::Patch),
                "missing patch version segment",
            ),
            (
                SemverErrorKind::MissingVersionSegment(Position::Major),
                "missing major version segment",
            ),
            (
                SemverErrorKind::MissingVersionSegment(Position::PreRelease),
                "missing pre-release version segment",
            ),
            (
                SemverErrorKind::MissingVersionSegment(Position::BuildMetadata),
                "missing build metadata version segment",
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
