#[cfg(not(feature = "std"))]
use alloc::format;

use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use crate::identifier::{BuildMetadata, PreRelease};
use crate::number::{MAX_SAFE_INTEGER, parse_core_number_digits};
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

    // Skip optional leading v prefix.
    let mut pos = usize::from(matches!(b.first(), Some(b'v')));

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
        PreRelease::new(pre_str)?
    } else {
        PreRelease::default()
    };

    // Optional build metadata
    let build = if b.get(pos) == Some(&b'+') {
        pos += 1;
        BuildMetadata::new(&raw[pos..])?
    } else if pos == b.len() {
        BuildMetadata::default()
    } else {
        return Err(SemverError::new(format!("unexpected character: {raw}")));
    };

    Ok(Version {
        major,
        minor,
        patch,
        pre_release,
        build,
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
