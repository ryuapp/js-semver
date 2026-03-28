#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, vec::Vec};

use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use crate::{MAX_LENGTH, MAX_SAFE_INTEGER, SemverError};

// --------------------------------------------------------------------------
// Identifier
// --------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum PreReleaseIdentifier {
    Numeric(Box<str>),
    AlphaNumeric(Box<str>),
}

impl fmt::Display for PreReleaseIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numeric(n) => f.write_str(n),
            Self::AlphaNumeric(s) => f.write_str(s),
        }
    }
}

impl PartialOrd for PreReleaseIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreReleaseIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Numeric(a), Self::Numeric(b)) => cmp_numeric_strings(a, b),
            (Self::Numeric(_), Self::AlphaNumeric(_)) => Ordering::Less,
            (Self::AlphaNumeric(_), Self::Numeric(_)) => Ordering::Greater,
            (Self::AlphaNumeric(a), Self::AlphaNumeric(b)) => a.cmp(b),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// A parsed pre-release identifier list such as `alpha.1`.
pub struct PreRelease(Box<[PreReleaseIdentifier]>);

impl PreRelease {
    pub(crate) fn zero() -> Self {
        Self(Box::from([PreReleaseIdentifier::Numeric(Box::from("0"))]))
    }

    /// Parse a pre-release identifier list such as `alpha.1`.
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not valid pre-release metadata.
    pub fn new(s: &str) -> Result<Self, SemverError> {
        parse_pre_release(s)
    }

    /// Parse a pre-release identifier list such as `alpha.1`.
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not valid pre-release metadata.
    pub fn parse(s: &str) -> Result<Self, SemverError> {
        Self::new(s)
    }

    #[must_use]
    /// Returns `true` when there are no pre-release identifiers.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn cmp_identifiers(&self, other: &Self) -> Ordering {
        for (left, right) in self.0.iter().zip(other.0.iter()) {
            match left.cmp(right) {
                Ordering::Equal => {}
                ord @ (Ordering::Less | Ordering::Greater) => return ord,
            }
        }
        self.0.len().cmp(&other.0.len())
    }
}

impl fmt::Display for PreRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, id) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            write!(f, "{id}")?;
        }
        Ok(())
    }
}

impl FromStr for PreRelease {
    type Err = SemverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Parsed build metadata such as `build.42`.
pub struct BuildMetadata(Box<[Box<str>]>);

impl BuildMetadata {
    /// Parse build metadata such as `build.42`.
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not valid build metadata.
    pub fn new(s: &str) -> Result<Self, SemverError> {
        parse_build_metadata(s)
    }

    /// Parse build metadata such as `build.42`.
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not valid build metadata.
    pub fn parse(s: &str) -> Result<Self, SemverError> {
        Self::new(s)
    }

    #[must_use]
    /// Returns `true` when there is no build metadata.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over build metadata identifiers.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(Box::as_ref)
    }

    pub(crate) const fn from_parts(parts: Box<[Box<str>]>) -> Self {
        Self(parts)
    }
}

impl fmt::Display for BuildMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, part) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            f.write_str(part)?;
        }
        Ok(())
    }
}

impl PartialOrd for BuildMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BuildMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        for (left, right) in self.0.iter().zip(other.0.iter()) {
            match cmp_build_identifier(left, right) {
                Ordering::Equal => {}
                ord @ (Ordering::Less | Ordering::Greater) => return ord,
            }
        }
        self.0.len().cmp(&other.0.len())
    }
}

impl FromStr for BuildMetadata {
    type Err = SemverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// --------------------------------------------------------------------------
// Version
// --------------------------------------------------------------------------

/// A parsed semantic version.
///
/// Build metadata is stored and participates in equality and total ordering.
#[derive(Debug, Clone, Eq)]
pub struct Version {
    /// The major version number.
    pub major: u64,
    /// The minor version number.
    pub minor: u64,
    /// The patch version number.
    pub patch: u64,
    /// The pre-release identifiers, if any.
    pub pre_release: PreRelease,
    /// The build metadata identifiers, if any.
    pub build: BuildMetadata,
}

impl Version {
    /// Create a new `Version` with no pre-release or build metadata.
    #[must_use]
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: PreRelease::default(),
            build: BuildMetadata::default(),
        }
    }

    /// Parse a version string.
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not a valid semver string.
    pub fn parse(s: &str) -> Result<Self, SemverError> {
        parse_version(s)
    }

    /// Compare semantic version precedence, ignoring build metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use core::cmp::Ordering;
    /// use js_semver::Version;
    ///
    /// let left: Version = "1.2.3+build.1".parse().unwrap();
    /// let right: Version = "1.2.3+build.2".parse().unwrap();
    ///
    /// assert_eq!(left.cmp_precedence(&right), Ordering::Equal);
    /// assert!(left < right);
    /// ```
    #[must_use]
    pub fn cmp_precedence(&self, other: &Self) -> Ordering {
        compare_core_and_prerelease(self, other)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.pre_release == other.pre_release
            && self.build == other.build
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.cmp_precedence(other) {
            Ordering::Equal => self.build.cmp(&other.build),
            ord @ (Ordering::Less | Ordering::Greater) => ord,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre_release.is_empty() {
            write!(f, "-{}", self.pre_release)?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = SemverError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_version(s)
    }
}

// --------------------------------------------------------------------------
// Version parsing (internal)
// --------------------------------------------------------------------------

fn parse_version(s: &str) -> Result<Version, SemverError> {
    let raw = s.trim();
    if raw.len() > MAX_LENGTH {
        return Err(SemverError::new("version string too long"));
    }
    let b = raw.as_bytes();

    // Skip optional v prefix then any trailing spaces (e.g. "v 1.2.3")
    let mut pos = usize::from(matches!(b.first(), Some(b'v')));
    while pos < b.len() && b[pos] == b' ' {
        pos += 1;
    }

    // Parse major.minor.patch in a single forward scan
    let major = parse_nr_at(b, &mut pos, raw)?;
    if b.get(pos) != Some(&b'.') {
        return Err(SemverError::new(format!("missing minor in: {raw}")));
    }
    pos += 1;
    let minor = parse_nr_at(b, &mut pos, raw)?;
    if b.get(pos) != Some(&b'.') {
        return Err(SemverError::new(format!("missing patch in: {raw}")));
    }
    pos += 1;
    let patch = parse_nr_at(b, &mut pos, raw)?;

    // Optional pre-release
    let pre_release = if b.get(pos) == Some(&b'-') {
        pos += 1;
        let start = pos;
        while pos < b.len() && b[pos] != b'+' {
            pos += 1;
        }
        let pre_str = &raw[start..pos];
        if pre_str.is_empty() {
            return Err(SemverError::new("empty pre-release"));
        }
        parse_pre_release(pre_str)?
    } else {
        PreRelease::default()
    };

    // Optional build metadata
    let build = if b.get(pos) == Some(&b'+') {
        pos += 1;
        parse_build_metadata(&raw[pos..])?.0
    } else if pos == b.len() {
        Box::default()
    } else {
        return Err(SemverError::new(format!("unexpected character: {raw}")));
    };

    Ok(Version {
        major,
        minor,
        patch,
        pre_release,
        build: BuildMetadata::from_parts(build),
    })
}

/// Parse a decimal integer from `b` starting at `*pos`, advancing `*pos` past the digits.
fn parse_nr_at(b: &[u8], pos: &mut usize, ctx: &str) -> Result<u64, SemverError> {
    let start = *pos;
    if start >= b.len() || !b[start].is_ascii_digit() {
        return Err(SemverError::new(format!("expected number in: {ctx}")));
    }
    // Leading-zero check
    if b[start] == b'0' && b.get(start + 1).is_some_and(u8::is_ascii_digit) {
        return Err(SemverError::new(format!("leading zero not allowed: {ctx}")));
    }
    while *pos < b.len() && b[*pos].is_ascii_digit() {
        *pos += 1;
    }
    let digits = &b[start..*pos];
    if digits.len() > 16 {
        return Err(SemverError::new(format!(
            "number exceeds MAX_SAFE_INTEGER: {}",
            &ctx[start..*pos]
        )));
    }
    let n = parse_core_number_digits(digits, ctx)?;
    if n > MAX_SAFE_INTEGER {
        return Err(SemverError::new(format!(
            "number exceeds MAX_SAFE_INTEGER: {n}"
        )));
    }
    Ok(n)
}

fn parse_core_number_digits(digits: &[u8], ctx: &str) -> Result<u64, SemverError> {
    let mut value = 0u64;
    for &digit in digits {
        if !digit.is_ascii_digit() {
            return Err(SemverError::new(format!("not a number: {ctx}")));
        }
        value = value * 10 + u64::from(digit - b'0');
    }
    Ok(value)
}

pub(crate) fn parse_nr(s: &str) -> Result<u64, SemverError> {
    let b = s.as_bytes();
    if b.is_empty() {
        return Err(SemverError::new("empty number"));
    }
    if b.len() > 1 && b[0] == b'0' {
        return Err(SemverError::new(format!("leading zero not allowed: {s}")));
    }
    // MAX_SAFE_INTEGER = 9_007_199_254_740_991 has 16 digits; 17+ digits always overflow.
    if b.len() > 16 {
        return Err(SemverError::new(format!(
            "number exceeds MAX_SAFE_INTEGER: {s}"
        )));
    }
    let mut n: u64 = 0;
    for &byte in b {
        if !byte.is_ascii_digit() {
            return Err(SemverError::new(format!("not a number: {s}")));
        }
        n = n * 10 + u64::from(byte - b'0');
    }
    if n > MAX_SAFE_INTEGER {
        return Err(SemverError::new(format!(
            "number exceeds MAX_SAFE_INTEGER: {n}"
        )));
    }
    Ok(n)
}

pub(crate) fn parse_pre_release(s: &str) -> Result<PreRelease, SemverError> {
    if s.is_empty() {
        return Err(SemverError::new("empty pre-release"));
    }
    let bytes = s.as_bytes();
    let mut ids = Vec::with_capacity(dot_separated_segments(bytes));
    let mut start = 0;
    let mut pos = 0;
    while pos <= bytes.len() {
        if pos == bytes.len() || bytes[pos] == b'.' {
            ids.push(parse_pre_id(&s[start..pos])?);
            start = pos + 1;
        }
        pos += 1;
    }
    Ok(PreRelease(ids.into_boxed_slice()))
}

fn parse_build_metadata(s: &str) -> Result<BuildMetadata, SemverError> {
    if s.is_empty() {
        return Err(SemverError::new("empty build metadata"));
    }
    let bytes = s.as_bytes();
    let mut parts = Vec::with_capacity(dot_separated_segments(bytes));
    let mut start = 0;
    let mut pos = 0;
    while pos <= bytes.len() {
        if pos == bytes.len() || bytes[pos] == b'.' {
            let id = &s[start..pos];
            if id.is_empty() {
                return Err(SemverError::new("empty build metadata identifier"));
            }
            if !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return Err(SemverError::new(format!(
                    "invalid build metadata identifier: {id}"
                )));
            }
            parts.push(Box::<str>::from(id));
            start = pos + 1;
        }
        pos += 1;
    }
    Ok(BuildMetadata(parts.into_boxed_slice()))
}

fn parse_pre_id(part: &str) -> Result<PreReleaseIdentifier, SemverError> {
    let b = part.as_bytes();
    if b.is_empty() {
        return Err(SemverError::new("empty pre-release identifier"));
    }
    // Single pass: validate chars and determine whether the identifier is fully numeric.
    let mut all_digits = true;
    for &byte in b {
        if byte.is_ascii_digit() {
        } else if byte.is_ascii_alphabetic() || byte == b'-' {
            all_digits = false;
        } else {
            return Err(SemverError::new(format!(
                "invalid pre-release identifier: {part}"
            )));
        }
    }
    if all_digits {
        if b.len() > 1 && b[0] == b'0' {
            return Err(SemverError::new(format!(
                "leading zero in pre-release: {part}"
            )));
        }
        Ok(PreReleaseIdentifier::Numeric(Box::from(part)))
    } else {
        Ok(PreReleaseIdentifier::AlphaNumeric(Box::from(part)))
    }
}

fn dot_separated_segments(bytes: &[u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while pos < bytes.len() {
        if bytes[pos] == b'.' {
            count += 1;
        }
        pos += 1;
    }
    count
}

fn cmp_numeric_strings(left: &str, right: &str) -> Ordering {
    match left.len().cmp(&right.len()) {
        Ordering::Equal => left.cmp(right),
        ord @ (Ordering::Less | Ordering::Greater) => ord,
    }
}

fn compare_core_and_prerelease(left: &Version, right: &Version) -> Ordering {
    macro_rules! cmp_field {
        ($field:ident) => {
            match left.$field.cmp(&right.$field) {
                Ordering::Equal => {}
                ord @ (Ordering::Less | Ordering::Greater) => return ord,
            }
        };
    }
    cmp_field!(major);
    cmp_field!(minor);
    cmp_field!(patch);
    match (left.pre_release.is_empty(), right.pre_release.is_empty()) {
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (true, true) => Ordering::Equal,
        (false, false) => left.pre_release.cmp_identifiers(&right.pre_release),
    }
}

fn cmp_build_identifier(left: &str, right: &str) -> Ordering {
    let left_is_numeric = left.bytes().all(|byte| byte.is_ascii_digit());
    let right_is_numeric = right.bytes().all(|byte| byte.is_ascii_digit());
    match (left_is_numeric, right_is_numeric) {
        (true, true) => cmp_numeric_strings(left, right),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => left.cmp(right),
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::{
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    use super::*;

    fn v(s: &str) -> Version {
        s.parse().unwrap()
    }

    // --- Version parsing ---

    #[test]
    fn parse_valid_and_display_cases() {
        let basic = v("1.2.3");
        assert_eq!((basic.major, basic.minor, basic.patch), (1, 2, 3));
        assert!(basic.pre_release.is_empty());
        assert!(basic.build.is_empty());

        let with_pre = v("1.2.3-alpha.1");
        assert_eq!(with_pre.pre_release.to_string(), "alpha.1");

        let with_build = v("1.2.3+build.42");
        assert_eq!(
            with_build.build.iter().collect::<Vec<_>>(),
            vec!["build", "42"]
        );

        let cases = [
            ("1.2.3", "1.2.3"),
            ("1.2.3-alpha.1", "1.2.3-alpha.1"),
            ("1.2.3+build.42", "1.2.3"),
            ("1.2.3-alpha.1+build", "1.2.3-alpha.1"),
            ("v1.2.3", "1.2.3"),
            ("v 1.2.3", "1.2.3"),
            ("1.2.3--pre", "1.2.3--pre"),
            ("1.2.3-a+b", "1.2.3-a"),
            ("0.0.0", "0.0.0"),
            ("9007199254740991.0.0", "9007199254740991.0.0"),
            ("1.2.3-9007199254740992", "1.2.3-9007199254740992"),
            ("1.0.0-9007199254740992", "1.0.0-9007199254740992"),
            ("1.0.0-18446744073709551616", "1.0.0-18446744073709551616"),
        ];

        for (input, expected) in cases {
            assert_eq!(input.parse::<Version>().unwrap().to_string(), expected);
        }
    }

    #[test]
    fn build_participates_in_eq_and_ord() {
        assert_ne!(v("1.2.3+a"), v("1.2.3+b"));
        assert!(v("1.2.3+a") < v("1.2.3+b"));
        assert!(v("1.2.3+9") < v("1.2.3+a"));
        assert!(v("1.2.3+demo.90") < v("1.2.3+demo.090"));
    }

    #[test]
    fn cmp_precedence_ignores_build() {
        assert_eq!(v("1.2.3+a").cmp_precedence(&v("1.2.3+b")), Ordering::Equal);
        assert_eq!(
            v("1.2.3-alpha+meta.1").cmp_precedence(&v("1.2.3-alpha+meta.2")),
            Ordering::Equal
        );
    }

    // --- Comparison ---

    #[test]
    fn cmp_versions() {
        assert!(v("1.0.0") < v("2.0.0"));
        assert!(v("2.0.0") > v("1.0.0"));
        assert_eq!(v("1.0.0"), v("1.0.0"));
    }

    #[test]
    fn partial_ord_matches_total_order() {
        assert_eq!(v("1.0.0").partial_cmp(&v("2.0.0")), Some(Ordering::Less));
        assert_eq!(
            v("1.2.3+build.1").partial_cmp(&v("1.2.3+build.2")),
            Some(Ordering::Less)
        );
        assert_eq!(
            v("1.2.3-alpha").partial_cmp(&v("1.2.3-alpha")),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn pre_lower_than_release() {
        assert!(v("1.0.0-alpha") < v("1.0.0"));
        assert!(v("1.0.0-beta") > v("1.0.0-alpha"));
        assert!(v("1.0.0-1") < v("1.0.0-alpha")); // numeric < alphanum
    }

    #[test]
    fn comparators_gt_gte_lt_lte_eq_neq() {
        assert!(v("2.0.0") > v("1.0.0"));
        assert!(v("1.0.0") <= v("2.0.0"));
        assert!(v("1.0.0") >= v("1.0.0"));
        assert!(v("1.0.0") < v("2.0.0"));
        assert!(v("1.0.0") <= v("1.0.0"));
        assert_eq!(v("1.0.0"), v("1.0.0"));
        assert_ne!(v("1.0.0"), v("2.0.0"));
    }

    // --- sort ---

    #[test]
    fn sort_versions() {
        let mut vs: Vec<Version> = ["3.0.0", "1.0.0", "2.0.0", "2.0.0+demo.9", "2.0.0+demo.10"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        vs.sort();
        assert_eq!(
            vs,
            [
                v("1.0.0"),
                v("2.0.0"),
                v("2.0.0+demo.9"),
                v("2.0.0+demo.10"),
                v("3.0.0"),
            ]
        );
        vs.sort_by(|a, b| b.cmp(a));
        assert_eq!(
            vs,
            [
                v("3.0.0"),
                v("2.0.0+demo.10"),
                v("2.0.0+demo.9"),
                v("2.0.0"),
                v("1.0.0"),
            ]
        );
    }

    // --- pre field ---

    #[test]
    fn prerelease_field() {
        assert!(v("1.2.3").pre_release.is_empty());
        assert!(!v("1.2.3-alpha.1").pre_release.is_empty());
        assert_eq!(v("1.2.3-alpha.1").pre_release.to_string(), "alpha.1");
    }

    // --- Version::parse static method ---

    // --- Ord: release > pre-release ---

    #[test]
    fn release_greater_than_prerelease() {
        use core::cmp::Ordering;
        assert_eq!(v("1.0.0").cmp(&v("1.0.0-alpha")), Ordering::Greater);
    }

    // --- parse errors ---

    #[test]
    fn parse_invalid_cases() {
        let cases = [
            "1.2.3".repeat(60),
            "1".into(),
            "1.2".into(),
            "01.2.3".into(),
            "1.02.3".into(),
            "18446744073709551616.0.0".into(),
            "1.2.3-".into(),
            "1.2.3+".into(),
            "1.2.3+a..b".into(),
            "1.2.3+a!b".into(),
            "1.2.3 extra".into(),
            "1.2.3-.0".into(),
            "1.2.3-a!b".into(),
            "1.2.3-01".into(),
            String::new(),
            "   ".into(),
            "1.".into(),
            "1.2.".into(),
            "1..2.3".into(),
            "1.2.3.4".into(),
            "..1".into(),
            ".1.2".into(),
            "-1.2.3".into(),
            "1.2.3-pre..rel".into(),
            "1.2.3-0.1.".into(),
            "1.2.3-0.01".into(),
            "1.2.3-01.0".into(),
            "1.2.3-.".into(),
            "1.2.3++".into(),
            "1.2.3+.".into(),
            "1.9007199254740992.0".into(),
            "1.0.9007199254740992".into(),
            "1.2.03".into(),
            "1.00.3".into(),
            "1.2.3\t4".into(),
            "1.2.3/".into(),
            "1.2.3#1".into(),
            "a.b.c".into(),
            "abc".into(),
            "+1.2.3".into(),
            "1.+2.3".into(),
            "1.2.+3".into(),
            "1 .2.3".into(),
            "1. 2.3".into(),
            "1.2. 3".into(),
            "1.2.3- alpha".into(),
            "1.2.3 -alpha".into(),
            "00.0.0".into(),
            "0.00.0".into(),
            "0.0.00".into(),
            "1.2.3-00".into(),
            "1.2.3-0.00".into(),
            "1.2.3+a b".into(),
            "1.2.3+a/b".into(),
            "1.2.3+a.b.".into(),
            "1.2.3+.a.b".into(),
            "１.0.0".into(),
            "1.２.0".into(),
            "V1.2.3".into(),
            "bad".into(),
        ];

        for input in cases {
            assert!(input.parse::<Version>().is_err());
            assert!(Version::parse(&input).is_err());
        }
    }

    // --- SemverError Display ---

    #[test]
    fn semver_error_display() {
        let err = "bad".parse::<Version>().unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn prerelease_public_api() {
        assert!(PreRelease::default().is_empty());
        assert_eq!(PreRelease::new("alpha.1").unwrap().to_string(), "alpha.1");
        assert_eq!(PreRelease::parse("beta").unwrap().to_string(), "beta");
        assert_eq!("rc.1".parse::<PreRelease>().unwrap().to_string(), "rc.1");
        assert_eq!(PreRelease::zero().to_string(), "0");
    }

    #[test]
    fn build_metadata_public_api() {
        assert!(BuildMetadata::default().is_empty());
        assert_eq!(
            BuildMetadata::new("build.001").unwrap().to_string(),
            "build.001"
        );
        assert_eq!(
            BuildMetadata::parse("sha.abcdef")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec!["sha", "abcdef"]
        );
        assert_eq!(
            "meta.42".parse::<BuildMetadata>().unwrap().to_string(),
            "meta.42"
        );
        assert_eq!(
            BuildMetadata::from_parts(Box::from([Box::<str>::from("x"), Box::<str>::from("y")]))
                .to_string(),
            "x.y"
        );
        assert_eq!(
            BuildMetadata::parse("alpha")
                .unwrap()
                .partial_cmp(&BuildMetadata::parse("1").unwrap()),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn prerelease_identifier_ordering() {
        assert_eq!(
            PreReleaseIdentifier::Numeric(Box::from("1"))
                .cmp(&PreReleaseIdentifier::Numeric(Box::from("2"))),
            Ordering::Less
        );
        assert_eq!(
            PreReleaseIdentifier::Numeric(Box::from("18446744073709551615")).cmp(
                &PreReleaseIdentifier::Numeric(Box::from("18446744073709551616"))
            ),
            Ordering::Less
        );
        assert_eq!(
            PreReleaseIdentifier::Numeric(Box::from("18446744073709551616")).cmp(
                &PreReleaseIdentifier::Numeric(Box::from("18446744073709551617"))
            ),
            Ordering::Less
        );
        assert_eq!(
            PreReleaseIdentifier::Numeric(Box::from("1"))
                .cmp(&PreReleaseIdentifier::AlphaNumeric(Box::from("alpha"))),
            Ordering::Less
        );
        assert_eq!(
            PreReleaseIdentifier::AlphaNumeric(Box::from("beta"))
                .cmp(&PreReleaseIdentifier::Numeric(Box::from("1"))),
            Ordering::Greater
        );
        assert_eq!(
            PreReleaseIdentifier::AlphaNumeric(Box::from("alpha"))
                .cmp(&PreReleaseIdentifier::AlphaNumeric(Box::from("beta"))),
            Ordering::Less
        );
        assert_eq!(
            PreReleaseIdentifier::Numeric(Box::from("1"))
                .partial_cmp(&PreReleaseIdentifier::Numeric(Box::from("2"))),
            Some(Ordering::Less)
        );
    }

    #[test]
    fn parse_nr_api() {
        assert_eq!(parse_nr("0").unwrap(), 0);
        assert_eq!(parse_nr("9007199254740991").unwrap(), MAX_SAFE_INTEGER);
        assert!(parse_nr("").is_err());
        assert!(parse_nr("01").is_err());
        assert!(parse_nr("1a").is_err());
        assert!(parse_nr("9007199254740992").is_err());
        assert!(parse_nr("12345678901234567").is_err());
    }

    #[test]
    fn private_helpers_edge_cases() {
        assert!(parse_pre_release("").is_err());
        assert!(parse_core_number_digits(b"1a", "1a").is_err());
    }

    #[test]
    fn max_safe_integer_core_components() {
        assert!("9007199254740991.0.0".parse::<Version>().is_ok());
        assert!("9007199254740992.0.0".parse::<Version>().is_err());
        assert!("1.9007199254740991.0".parse::<Version>().is_ok());
        assert!("1.0.9007199254740991".parse::<Version>().is_ok());
        assert!("1.9007199254740992.0".parse::<Version>().is_err());
        assert!("1.0.9007199254740992".parse::<Version>().is_err());
    }
}
