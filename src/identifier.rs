#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use crate::SemverError;
use crate::error::SemverErrorKind;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// A parsed pre-release identifier list such as `alpha.1`.
pub struct PreRelease(Box<str>);

impl PreRelease {
    pub(crate) fn zero() -> Self {
        Self(Box::from("0"))
    }

    /// Parse a pre-release identifier list such as `alpha.1`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::PreRelease;
    ///
    /// let pre = PreRelease::new("alpha.1").unwrap();
    ///
    /// assert_eq!(pre.to_string(), "alpha.1");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not valid pre-release metadata.
    pub fn new(s: &str) -> Result<Self, SemverError> {
        if s.is_empty() {
            return Err(SemverErrorKind::Empty.into());
        }
        validate_prerelease(s)?;
        Ok(Self(Box::from(s)))
    }

    #[must_use]
    /// Returns `true` when there are no pre-release identifiers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::PreRelease;
    ///
    /// assert!(PreRelease::default().is_empty());
    /// assert!(!PreRelease::new("rc.1").unwrap().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn cmp_identifiers(&self, other: &Self) -> Ordering {
        if self.is_empty() || other.is_empty() {
            return self.0.len().cmp(&other.0.len());
        }

        cmp_dot_separated(&self.0, &other.0, parse_prerelease_identifier)
    }
}

impl fmt::Display for PreRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialOrd for PreRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_identifiers(other)
    }
}

impl FromStr for PreRelease {
    type Err = SemverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Parsed build metadata such as `build.42`.
pub struct BuildMetadata(Box<str>);

impl BuildMetadata {
    /// Parse build metadata such as `build.42`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::BuildMetadata;
    ///
    /// let build = BuildMetadata::new("build.42").unwrap();
    ///
    /// assert_eq!(build.to_string(), "build.42");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not valid build metadata.
    pub fn new(s: &str) -> Result<Self, SemverError> {
        if s.is_empty() {
            return Err(SemverErrorKind::Empty.into());
        }
        validate_build_metadata(s)?;
        Ok(Self(Box::from(s)))
    }

    #[must_use]
    /// Returns `true` when there is no build metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::BuildMetadata;
    ///
    /// assert!(BuildMetadata::default().is_empty());
    /// assert!(!BuildMetadata::new("sha.abcdef").unwrap().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for BuildMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialOrd for BuildMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BuildMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_empty() || other.is_empty() {
            return self.0.len().cmp(&other.0.len());
        }

        cmp_dot_separated(&self.0, &other.0, parse_build_metadata_identifier)
    }
}

impl FromStr for BuildMetadata {
    type Err = SemverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierKind {
    Numeric,
    AlphaNumeric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Identifier<'a> {
    raw: &'a str,
    kind: IdentifierKind,
}

impl<'a> Identifier<'a> {
    fn new(raw: &'a str, kind: IdentifierKind) -> Self {
        Self { raw, kind }
    }
}

fn parse_prerelease_identifier(raw: &str) -> Identifier<'_> {
    let bytes = raw.as_bytes();
    Identifier::new(
        raw,
        if bytes.iter().all(u8::is_ascii_digit) {
            IdentifierKind::Numeric
        } else {
            IdentifierKind::AlphaNumeric
        },
    )
}

fn parse_build_metadata_identifier(raw: &str) -> Identifier<'_> {
    Identifier::new(
        raw,
        if raw.bytes().all(|byte| byte.is_ascii_digit()) {
            IdentifierKind::Numeric
        } else {
            IdentifierKind::AlphaNumeric
        },
    )
}

impl PartialOrd for Identifier<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.kind, other.kind) {
            (IdentifierKind::Numeric, IdentifierKind::Numeric) => {
                cmp_numeric_strings(self.raw, other.raw)
            }
            (IdentifierKind::Numeric, IdentifierKind::AlphaNumeric) => Ordering::Less,
            (IdentifierKind::AlphaNumeric, IdentifierKind::Numeric) => Ordering::Greater,
            (IdentifierKind::AlphaNumeric, IdentifierKind::AlphaNumeric) => self.raw.cmp(other.raw),
        }
    }
}

fn validate_prerelease(s: &str) -> Result<(), SemverError> {
    let bytes = s.as_bytes();
    let mut segment_start = 0;
    let mut all_digits = true;

    for (pos, &byte) in bytes.iter().enumerate() {
        match byte {
            b'.' => {
                validate_prerelease_segment(bytes, segment_start, pos, all_digits)?;
                segment_start = pos + 1;
                all_digits = true;
            }
            b'0'..=b'9' => {}
            b'A'..=b'Z' | b'a'..=b'z' | b'-' => all_digits = false,
            _ => {
                return Err(SemverErrorKind::UnexpectedCharacter(char::from(byte)).into());
            }
        }
    }

    validate_prerelease_segment(bytes, segment_start, bytes.len(), all_digits)
}

fn validate_prerelease_segment(
    bytes: &[u8],
    start: usize,
    end: usize,
    all_digits: bool,
) -> Result<(), SemverError> {
    if start == end {
        return Err(SemverErrorKind::EmptySegment.into());
    }
    if all_digits && end - start > 1 && bytes[start] == b'0' {
        return Err(SemverErrorKind::LeadingZero.into());
    }
    Ok(())
}

fn validate_build_metadata(s: &str) -> Result<(), SemverError> {
    let bytes = s.as_bytes();
    let mut segment_start = 0;

    for (pos, &byte) in bytes.iter().enumerate() {
        match byte {
            b'.' => {
                if pos == segment_start {
                    return Err(SemverErrorKind::EmptySegment.into());
                }
                segment_start = pos + 1;
            }
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' => {}
            _ => {
                return Err(SemverErrorKind::UnexpectedCharacter(char::from(byte)).into());
            }
        }
    }

    if segment_start == bytes.len() {
        return Err(SemverErrorKind::EmptySegment.into());
    }
    Ok(())
}

fn cmp_dot_separated<'a>(
    left: &'a str,
    right: &'a str,
    parser: fn(&'a str) -> Identifier<'a>,
) -> Ordering {
    let mut left_start = 0;
    let mut right_start = 0;
    loop {
        let left_end = next_separator(left, left_start);
        let right_end = next_separator(right, right_start);
        let left_part = &left[left_start..left_end];
        let right_part = &right[right_start..right_end];
        let left_id = parser(left_part);
        let right_id = parser(right_part);
        match left_id.cmp(&right_id) {
            Ordering::Equal => {}
            ord @ (Ordering::Less | Ordering::Greater) => return ord,
        }
        let left_done = left_end == left.len();
        let right_done = right_end == right.len();
        if left_done {
            return if right_done {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        }
        if right_done {
            return Ordering::Greater;
        }
        left_start = left_end + 1;
        right_start = right_end + 1;
    }
}

fn cmp_numeric_strings(left: &str, right: &str) -> Ordering {
    match left.len().cmp(&right.len()) {
        Ordering::Equal => left.cmp(right),
        ord @ (Ordering::Less | Ordering::Greater) => ord,
    }
}

fn next_separator(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    let mut pos = start;
    while pos < bytes.len() && bytes[pos] != b'.' {
        pos += 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn identifier_ordering() {
        assert_eq!(
            parse_build_metadata_identifier("1").cmp(&parse_build_metadata_identifier("2")),
            Ordering::Less
        );
        assert_eq!(
            parse_build_metadata_identifier("18446744073709551615")
                .cmp(&parse_build_metadata_identifier("18446744073709551616")),
            Ordering::Less
        );
        assert_eq!(
            parse_build_metadata_identifier("18446744073709551616")
                .cmp(&parse_build_metadata_identifier("18446744073709551617")),
            Ordering::Less
        );
        assert_eq!(
            parse_prerelease_identifier("1").cmp(&parse_prerelease_identifier("alpha")),
            Ordering::Less
        );
        assert_eq!(
            parse_prerelease_identifier("beta").cmp(&parse_prerelease_identifier("1")),
            Ordering::Greater
        );
    }

    #[test]
    fn prerelease_identifier_validation() {
        assert!(PreRelease::new("").is_err());
        assert!(PreRelease::new("01").is_err());
        assert!(PreRelease::new("a!b").is_err());
        assert_eq!(
            parse_prerelease_identifier("alpha-1").kind,
            IdentifierKind::AlphaNumeric
        );
    }

    #[test]
    fn prerelease_public_api() {
        assert!(PreRelease::default().is_empty());
        assert!(PreRelease::new("").is_err());
        assert_eq!(PreRelease::new("alpha.1").unwrap().to_string(), "alpha.1");
        assert_eq!(PreRelease::new("beta").unwrap().to_string(), "beta");
        assert_eq!("rc.1".parse::<PreRelease>().unwrap().to_string(), "rc.1");
        assert_eq!(PreRelease::zero().to_string(), "0");
        assert!(PreRelease::new("alpha").unwrap() < PreRelease::new("beta").unwrap());
        assert!(PreRelease::new("1").unwrap() < PreRelease::new("alpha").unwrap());
        assert_eq!(
            PreRelease::new("alpha.1")
                .unwrap()
                .partial_cmp(&PreRelease::new("alpha.1").unwrap()),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn build_metadata_public_api() {
        assert!(BuildMetadata::default().is_empty());
        assert_eq!(
            BuildMetadata::new("build.001").unwrap().to_string(),
            "build.001"
        );
        assert_eq!(
            BuildMetadata::new("sha.abcdef").unwrap().to_string(),
            "sha.abcdef"
        );
        assert_eq!(
            "meta.42".parse::<BuildMetadata>().unwrap().to_string(),
            "meta.42"
        );
        assert_eq!(BuildMetadata::new("x.y").unwrap().to_string(), "x.y");
        assert_eq!(
            BuildMetadata::new("alpha")
                .unwrap()
                .partial_cmp(&BuildMetadata::new("1").unwrap()),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn empty_component_shortcuts() {
        assert_eq!(
            PreRelease::default().cmp(&PreRelease::zero()),
            Ordering::Less
        );
        assert_eq!(
            PreRelease::zero().cmp(&PreRelease::default()),
            Ordering::Greater
        );
        assert_eq!(
            BuildMetadata::default().cmp(&BuildMetadata::new("meta").unwrap()),
            Ordering::Less
        );
        assert_eq!(
            BuildMetadata::new("meta")
                .unwrap()
                .cmp(&BuildMetadata::default()),
            Ordering::Greater
        );
    }

    #[test]
    fn identifier_partial_cmp_and_prefix_order() {
        assert_eq!(
            parse_prerelease_identifier("alpha").partial_cmp(&parse_prerelease_identifier("alpha")),
            Some(Ordering::Equal)
        );
        assert_eq!(
            cmp_dot_separated("alpha", "alpha.1", parse_prerelease_identifier),
            Ordering::Less
        );
        assert_eq!(
            Ord::cmp(
                &PreRelease::new("alpha").unwrap(),
                &PreRelease::new("beta").unwrap()
            ),
            Ordering::Less
        );
        assert_eq!(
            PartialOrd::partial_cmp(
                &BuildMetadata::new("build.1").unwrap(),
                &BuildMetadata::new("build.2").unwrap()
            ),
            Some(Ordering::Less)
        );
        assert_eq!(
            parse_build_metadata_identifier("1").partial_cmp(&parse_build_metadata_identifier("2")),
            Some(Ordering::Less)
        );
        assert_eq!("rc.1".parse::<PreRelease>().unwrap().to_string(), "rc.1");
        assert_eq!(
            "meta.42".parse::<BuildMetadata>().unwrap().to_string(),
            "meta.42"
        );
    }

    #[test]
    fn prerelease_cmp_identifiers_covers_empty_and_equal_cases() {
        assert_eq!(
            PreRelease::default().cmp(&PreRelease::default()),
            Ordering::Equal
        );
        assert_eq!(
            PreRelease::default().cmp(&PreRelease::new("0").unwrap()),
            Ordering::Less
        );
        assert_eq!(
            PreRelease::new("alpha")
                .unwrap()
                .cmp_identifiers(&PreRelease::new("alpha").unwrap()),
            Ordering::Equal
        );
        assert!(PreRelease::new("").is_err());
        assert!(PreRelease::new("alpha!1").is_err());
        assert!(BuildMetadata::new("").is_err());
        assert!(BuildMetadata::new("meta!1").is_err());
    }
}
