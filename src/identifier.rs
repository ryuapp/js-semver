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
        validate_identifiers(s, parse_prerelease_identifier)?;
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
        validate_identifiers(s, parse_build_metadata_identifier)?;
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

fn parse_prerelease_identifier(raw: &str) -> Result<Identifier<'_>, SemverError> {
    let bytes = raw.as_bytes();
    if bytes.is_empty() {
        return Err(SemverErrorKind::Empty.into());
    }

    let mut all_digits = true;
    for &byte in bytes {
        if byte.is_ascii_digit() {
        } else if byte.is_ascii_alphabetic() || byte == b'-' {
            all_digits = false;
        } else {
            return Err(SemverErrorKind::UnexpectedCharacter(char::from(byte)).into());
        }
    }

    if all_digits && bytes.len() > 1 && bytes[0] == b'0' {
        return Err(SemverErrorKind::LeadingZero.into());
    }

    Ok(Identifier::new(
        raw,
        if all_digits {
            IdentifierKind::Numeric
        } else {
            IdentifierKind::AlphaNumeric
        },
    ))
}

fn parse_build_metadata_identifier(raw: &str) -> Result<Identifier<'_>, SemverError> {
    if raw.is_empty() {
        return Err(SemverErrorKind::Empty.into());
    }
    if let Some(byte) = raw
        .bytes()
        .find(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-')
    {
        return Err(SemverErrorKind::UnexpectedCharacter(char::from(byte)).into());
    }
    Ok(Identifier::new(
        raw,
        if raw.bytes().all(|byte| byte.is_ascii_digit()) {
            IdentifierKind::Numeric
        } else {
            IdentifierKind::AlphaNumeric
        },
    ))
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

fn validate_identifiers<'a>(
    s: &'a str,
    parser: fn(&'a str) -> Result<Identifier<'a>, SemverError>,
) -> Result<(), SemverError> {
    let mut start = 0;
    while start <= s.len() {
        let end = next_separator(s, start);
        parser(&s[start..end])?;
        if end == s.len() {
            break;
        }
        start = end + 1;
    }
    Ok(())
}

fn cmp_dot_separated<'a>(
    left: &'a str,
    right: &'a str,
    parser: fn(&'a str) -> Result<Identifier<'a>, SemverError>,
) -> Ordering {
    let mut left_start = 0;
    let mut right_start = 0;
    loop {
        let left_end = next_separator(left, left_start);
        let right_end = next_separator(right, right_start);
        let left_part = &left[left_start..left_end];
        let right_part = &right[right_start..right_end];
        let Ok(left_id) = parser(left_part) else {
            return Ordering::Equal;
        };
        let Ok(right_id) = parser(right_part) else {
            return Ordering::Equal;
        };
        match left_id.cmp(&right_id) {
            Ordering::Equal => {}
            ord @ (Ordering::Less | Ordering::Greater) => return ord,
        }
        let left_done = left_end == left.len();
        let right_done = right_end == right.len();
        if left_done || right_done {
            return match (left_done, right_done) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (true, true) | (false, false) => Ordering::Equal,
            };
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
            parse_build_metadata_identifier("1")
                .unwrap()
                .cmp(&parse_build_metadata_identifier("2").unwrap()),
            Ordering::Less
        );
        assert_eq!(
            parse_build_metadata_identifier("18446744073709551615")
                .unwrap()
                .cmp(&parse_build_metadata_identifier("18446744073709551616").unwrap()),
            Ordering::Less
        );
        assert_eq!(
            parse_build_metadata_identifier("18446744073709551616")
                .unwrap()
                .cmp(&parse_build_metadata_identifier("18446744073709551617").unwrap()),
            Ordering::Less
        );
        assert_eq!(
            parse_prerelease_identifier("1")
                .unwrap()
                .cmp(&parse_prerelease_identifier("alpha").unwrap()),
            Ordering::Less
        );
        assert_eq!(
            parse_prerelease_identifier("beta")
                .unwrap()
                .cmp(&parse_prerelease_identifier("1").unwrap()),
            Ordering::Greater
        );
    }

    #[test]
    fn prerelease_identifier_validation() {
        assert!(parse_prerelease_identifier("").is_err());
        assert!(parse_prerelease_identifier("01").is_err());
        assert!(parse_prerelease_identifier("a!b").is_err());
        assert_eq!(
            parse_prerelease_identifier("alpha-1").unwrap().kind,
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
            parse_prerelease_identifier("alpha")
                .unwrap()
                .partial_cmp(&parse_prerelease_identifier("alpha").unwrap()),
            Some(Ordering::Equal)
        );
        assert_eq!(
            cmp_dot_separated("alpha", "alpha.1", parse_prerelease_identifier),
            Ordering::Less
        );
    }
}
