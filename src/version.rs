use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use crate::error::{Position, SemverErrorKind};
use crate::identifier::{BuildMetadata, PreRelease};
use crate::number::{MAX_SAFE_INTEGER, MAX_SAFE_INTEGER_DIGITS};
use crate::{MAX_LENGTH, SemverError};

// --------------------------------------------------------------------------
// Version
// --------------------------------------------------------------------------

/// A parsed semantic version.
///
/// Build metadata is stored and included in the version's string form.
/// Direct [`Version`] comparison ignores build metadata.
/// Use [`Version::cmp_build`] when build metadata should be used as a
/// tiebreaker.
///
/// # Examples
///
/// ```rust
/// use js_semver::Version;
///
/// let version = Version::parse("19.3.0-canary-044d56f3-20260330").unwrap();
///
/// assert_eq!(version.major, 19);
/// assert_eq!(version.minor, 3);
/// assert_eq!(version.patch, 0);
/// assert_eq!(version.to_string(), "19.3.0-canary-044d56f3-20260330");
/// ```
#[derive(Debug, Clone, Eq)]
pub struct Version {
    /// The major version.
    pub major: u64,
    /// The minor version.
    pub minor: u64,
    /// The patch version.
    pub patch: u64,
    /// The pre-release identifiers, if any.
    pub pre_release: PreRelease,
    /// The build metadata identifiers, if any.
    pub build: BuildMetadata,
}

impl Version {
    /// Create a new `Version` with no pre-release or build metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::Version;
    ///
    /// let version = Version::new(1, 2, 3);
    ///
    /// assert_eq!(version.to_string(), "1.2.3");
    /// assert!(version.pre_release.is_empty());
    /// assert!(version.build.is_empty());
    /// ```
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
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::Version;
    ///
    /// let version = Version::parse("1.2.3-alpha.1").unwrap();
    ///
    /// assert_eq!(version.major, 1);
    /// assert_eq!(version.minor, 2);
    /// assert_eq!(version.patch, 3);
    /// assert_eq!(version.pre_release.to_string(), "alpha.1");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not a valid semver string.
    pub fn parse(s: &str) -> Result<Self, SemverError> {
        if s.len() > MAX_LENGTH {
            return Err(SemverErrorKind::MaxLengthExceeded.into());
        }
        if let Some(version) = parse_fixed_core_version(s.as_bytes()) {
            return Ok(version);
        }
        let ascii_trimmed = trim_ascii_whitespace(s);
        if ascii_trimmed.len() != s.len() {
            if let Some(version) = parse_fixed_core_version(ascii_trimmed.as_bytes()) {
                return Ok(version);
            }
        }
        parse_version(s)
    }

    /// Compare semantic version precedence with build metadata as a tiebreaker.
    ///
    /// This is equivalent to `node-semver`'s `compareBuild()`.
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
    /// assert_eq!(left.cmp(&right), Ordering::Equal);
    /// assert_eq!(left.cmp_build(&right), Ordering::Less);
    /// ```
    #[must_use]
    pub fn cmp_build(&self, other: &Self) -> Ordering {
        match compare_core_and_prerelease(self, other) {
            Ordering::Equal => self.build.cmp(&other.build),
            ord @ (Ordering::Less | Ordering::Greater) => ord,
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.pre_release == other.pre_release
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_core_and_prerelease(self, other)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre_release.is_empty() {
            write!(f, "-{}", self.pre_release)?;
        }
        if !self.build.is_empty() {
            write!(f, "+{}", self.build)?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = SemverError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// --------------------------------------------------------------------------
// Version parsing (internal)
// --------------------------------------------------------------------------

fn parse_version(s: &str) -> Result<Version, SemverError> {
    if let Some(version) = parse_fast_version(s) {
        return Ok(version);
    }
    let ascii_trimmed = trim_ascii_whitespace(s);
    if ascii_trimmed.len() != s.len() {
        if let Some(version) = parse_fast_version(ascii_trimmed) {
            return Ok(version);
        }
    }
    let raw = s.trim();
    if raw.is_empty() {
        return match s.chars().next() {
            Some(unexpected) => Err(SemverErrorKind::UnexpectedCharacterWhileParsing(
                unexpected,
                Position::Major,
            )
            .into()),
            None => Err(SemverErrorKind::Empty.into()),
        };
    }
    let b = raw.as_bytes();

    // Skip optional leading v prefix.
    let mut pos = usize::from(matches!(b.first(), Some(b'v')));

    // Parse major.minor.patch in a single forward scan
    let major = parse_nr_at(raw, &mut pos, Position::Major)?;
    if b.get(pos) != Some(&b'.') {
        return Err(SemverErrorKind::MissingVersionSegment(Position::Minor).into());
    }
    pos += 1;
    let minor = parse_nr_at(raw, &mut pos, Position::Minor)?;
    if b.get(pos) != Some(&b'.') {
        return Err(SemverErrorKind::MissingVersionSegment(Position::Patch).into());
    }
    pos += 1;
    let patch = parse_nr_at(raw, &mut pos, Position::Patch)?;

    // Optional pre-release
    let pre_release = if b.get(pos) == Some(&b'-') {
        pos += 1;
        let start = pos;
        while pos < b.len() && b[pos] != b'+' {
            pos += 1;
        }
        let pre_str = &raw[start..pos];
        if pre_str.is_empty() {
            return Err(SemverErrorKind::EmptyIdentifierSegment(Position::PreRelease).into());
        }
        PreRelease::new(pre_str)?
    } else {
        PreRelease::default()
    };

    // Optional build metadata
    let build = if b.get(pos) == Some(&b'+') {
        pos += 1;
        BuildMetadata::new(&raw[pos..])?
    } else {
        match raw[pos..].chars().next() {
            Some(unexpected) => {
                return Err(
                    SemverErrorKind::UnexpectedCharacterAfter(unexpected, Position::Patch).into(),
                );
            }
            None => BuildMetadata::default(),
        }
    };

    Ok(Version {
        major,
        minor,
        patch,
        pre_release,
        build,
    })
}

fn parse_fixed_core_version(bytes: &[u8]) -> Option<Version> {
    let pos = usize::from(matches!(bytes.first(), Some(b'v')));
    if bytes.len() - pos == 5
        && bytes[pos].is_ascii_digit()
        && bytes[pos + 1] == b'.'
        && bytes[pos + 2].is_ascii_digit()
        && bytes[pos + 3] == b'.'
        && bytes[pos + 4].is_ascii_digit()
    {
        return Some(Version::new(
            u64::from(bytes[pos] - b'0'),
            u64::from(bytes[pos + 2] - b'0'),
            u64::from(bytes[pos + 4] - b'0'),
        ));
    }

    None
}

fn parse_fast_version(input: &str) -> Option<Version> {
    let bytes = input.as_bytes();
    let mut pos = usize::from(matches!(bytes.first(), Some(b'v')));
    let major = parse_simple_core_number(bytes, &mut pos)?;
    if bytes.get(pos) != Some(&b'.') {
        return None;
    }
    pos += 1;

    let minor = parse_simple_core_number(bytes, &mut pos)?;
    if bytes.get(pos) != Some(&b'.') {
        return None;
    }
    pos += 1;

    let patch = parse_simple_core_number(bytes, &mut pos)?;
    if pos == bytes.len() {
        return Some(Version::new(major, minor, patch));
    }

    let pre_release = if bytes.get(pos) == Some(&b'-') {
        pos += 1;
        let start = pos;
        while pos < bytes.len() && bytes[pos] != b'+' {
            pos += 1;
        }
        if start == pos {
            return None;
        }
        PreRelease::new(&input[start..pos]).ok()?
    } else {
        PreRelease::default()
    };

    let build = if bytes.get(pos) == Some(&b'+') {
        BuildMetadata::new(&input[pos + 1..]).ok()?
    } else if pos == bytes.len() {
        BuildMetadata::default()
    } else {
        return None;
    };

    Some(Version {
        major,
        minor,
        patch,
        pre_release,
        build,
    })
}

fn trim_ascii_whitespace(input: &str) -> &str {
    let bytes = input.as_bytes();
    let mut start = 0;
    while bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
        start += 1;
    }

    let mut end = bytes.len();
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &input[start..end]
}

fn parse_simple_core_number(bytes: &[u8], pos: &mut usize) -> Option<u64> {
    let start = *pos;
    let first = bytes.get(start).copied()?;
    if !first.is_ascii_digit()
        || (first == b'0' && bytes.get(start + 1).is_some_and(u8::is_ascii_digit))
    {
        return None;
    }

    let mut value = 0u64;
    while let Some(digit @ b'0'..=b'9') = bytes.get(*pos).copied() {
        if *pos - start == MAX_SAFE_INTEGER_DIGITS {
            return None;
        }
        value = value * 10 + u64::from(digit - b'0');
        *pos += 1;
    }
    (value <= MAX_SAFE_INTEGER).then_some(value)
}

/// Parse a decimal integer from `input` starting at `*pos`, advancing past the digits.
fn parse_nr_at(input: &str, pos: &mut usize, position: Position) -> Result<u64, SemverError> {
    let b = input.as_bytes();
    let start = *pos;
    if start >= b.len() {
        return Err(SemverErrorKind::MissingVersionSegment(position).into());
    }
    if !b[start].is_ascii_digit() {
        let Some(unexpected) = input.get(start..).and_then(|tail| tail.chars().next()) else {
            return Err(SemverErrorKind::MissingVersionSegment(position).into());
        };
        return Err(SemverErrorKind::UnexpectedCharacterWhileParsing(unexpected, position).into());
    }
    // Leading-zero check
    if b[start] == b'0' && b.get(start + 1).is_some_and(u8::is_ascii_digit) {
        return Err(SemverErrorKind::LeadingZero(position).into());
    }
    let mut value = 0u64;
    while let Some(&digit) = b.get(*pos).filter(|digit| digit.is_ascii_digit()) {
        if *pos - start == MAX_SAFE_INTEGER_DIGITS {
            return Err(SemverErrorKind::MaxSafeIntegerExceeded(position).into());
        }
        value = value * 10 + u64::from(digit - b'0');
        *pos += 1;
    }
    if value > MAX_SAFE_INTEGER {
        return Err(SemverErrorKind::MaxSafeIntegerExceeded(position).into());
    }
    Ok(value)
}

pub(crate) fn compare_core_and_prerelease(left: &Version, right: &Version) -> Ordering {
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

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    use super::{Version, parse_nr_at};
    use crate::error::Position;
    use crate::number::MAX_SAFE_INTEGER;

    #[test]
    fn parse_nr_at_propagates_core_number_parse_errors() {
        let input = "9007199254740992";
        let mut pos = 0;
        assert!(parse_nr_at(input, &mut pos, Position::Major).is_err());

        let mut invalid_utf8_boundary = 1;
        assert!(parse_nr_at("é", &mut invalid_utf8_boundary, Position::Major).is_err());
    }

    #[test]
    fn parse_nr_at_parses_max_safe_integer() {
        let input = "9007199254740991";
        let mut pos = 0;
        assert_eq!(
            parse_nr_at(input, &mut pos, Position::Major).unwrap(),
            MAX_SAFE_INTEGER
        );
        assert_eq!(pos, input.len());
    }

    #[test]
    fn parse_covers_trimmed_fast_and_unicode_fallback_paths() {
        assert_eq!(Version::parse(" 1.2.3 ").unwrap(), Version::new(1, 2, 3));
        assert_eq!(Version::parse(" 10.2.3 ").unwrap(), Version::new(10, 2, 3));
        assert_eq!(
            Version::parse("\u{2003}1.2.3\u{2003}").unwrap(),
            Version::new(1, 2, 3)
        );
    }

    #[test]
    fn core_version_errors_include_the_position() {
        assert_eq!(
            Version::parse("1..3").unwrap_err().to_string(),
            "unexpected character '.' while parsing minor version"
        );
        assert_eq!(
            Version::parse("1.a.3").unwrap_err().to_string(),
            "unexpected character 'a' while parsing minor version"
        );
        assert_eq!(
            Version::parse("1.2.a").unwrap_err().to_string(),
            "unexpected character 'a' while parsing patch version"
        );
        assert_eq!(
            Version::parse("1").unwrap_err().to_string(),
            "missing minor version segment"
        );
        assert_eq!(
            Version::parse("1.2").unwrap_err().to_string(),
            "missing patch version segment"
        );
        assert_eq!(
            Version::parse("01.2.3").unwrap_err().to_string(),
            "invalid leading zero in major version"
        );
        assert_eq!(
            Version::parse("1.02.3").unwrap_err().to_string(),
            "invalid leading zero in minor version"
        );
        assert_eq!(
            Version::parse("1.2.03").unwrap_err().to_string(),
            "invalid leading zero in patch version"
        );
        assert_eq!(
            Version::parse("9007199254740992.0.0")
                .unwrap_err()
                .to_string(),
            "number exceeds MAX_SAFE_INTEGER in major version"
        );
        assert_eq!(
            Version::parse("1.2.3!").unwrap_err().to_string(),
            "unexpected character '!' after patch version"
        );
    }

    #[test]
    fn empty_error_is_only_used_for_an_empty_version_input() {
        assert_eq!(Version::parse("").unwrap_err().to_string(), "empty");
        assert_eq!(
            Version::parse("   ").unwrap_err().to_string(),
            "unexpected character ' ' while parsing major version"
        );
        assert_eq!(
            Version::parse("1.2.3-").unwrap_err().to_string(),
            "empty identifier segment in pre-release identifier"
        );
        assert_eq!(
            Version::parse("1.2.3+").unwrap_err().to_string(),
            "empty identifier segment in build metadata"
        );
        assert_eq!(
            Version::parse("1.2.3-alpha..1").unwrap_err().to_string(),
            "empty identifier segment in pre-release identifier"
        );
        assert_eq!(
            Version::parse("1.2.3+build..1").unwrap_err().to_string(),
            "empty identifier segment in build metadata"
        );
    }
}
