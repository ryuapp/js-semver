#[cfg(not(feature = "std"))]
use alloc::{format, vec, vec::Vec};

use core::fmt;
use core::str::FromStr;

use crate::identifier::{BuildMetadata, PreRelease};
use crate::version::{Version, parse_nr};
use crate::{MAX_LENGTH, SemverError};

// --------------------------------------------------------------------------
// Range types
// --------------------------------------------------------------------------

/// Comparison operator used in a version comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    /// `<` — less than.
    LessThan,
    /// `<=` — less than or equal to.
    LessThanOrEqual,
    /// `>` — greater than.
    GreaterThan,
    /// `>=` — greater than or equal to.
    GreaterThanOrEqual,
    /// `=` — exactly equal.
    Equal,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::Equal => "=",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Comparator {
    op: Operator,
    version: Version,
}

impl Comparator {
    fn test(&self, v: &Version) -> bool {
        let cmp = compare_core_and_prerelease(v, &self.version);
        match self.op {
            Operator::Equal => cmp == core::cmp::Ordering::Equal,
            Operator::GreaterThan => cmp == core::cmp::Ordering::Greater,
            Operator::GreaterThanOrEqual => cmp != core::cmp::Ordering::Less,
            Operator::LessThan => cmp == core::cmp::Ordering::Less,
            Operator::LessThanOrEqual => cmp != core::cmp::Ordering::Greater,
        }
    }
}

impl fmt::Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op {
            Operator::Equal => write!(f, "{}", self.version),
            Operator::LessThan
            | Operator::LessThanOrEqual
            | Operator::GreaterThan
            | Operator::GreaterThanOrEqual => write!(f, "{}{}", self.op, self.version),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComparatorSet {
    comparators: Vec<Comparator>,
}

impl ComparatorSet {
    fn test(&self, v: &Version) -> bool {
        if self.comparators.is_empty() {
            return v.pre_release.is_empty();
        }

        if v.pre_release.is_empty() {
            for comparator in &self.comparators {
                if !comparator.test(v) {
                    return false;
                }
            }
            return true;
        }

        let mut has_matching_prerelease_tuple = false;
        for comparator in &self.comparators {
            if !comparator.test(v) {
                return false;
            }
            let comparator_version = &comparator.version;
            if !comparator_version.pre_release.is_empty()
                && comparator_version.major == v.major
                && comparator_version.minor == v.minor
                && comparator_version.patch == v.patch
            {
                has_matching_prerelease_tuple = true;
            }
        }
        has_matching_prerelease_tuple
    }
}

/// A version range, e.g. `^1.0.0` or `>=1.0.0 <2.0.0-0`.
///
/// Its string form is canonicalized, so `to_string()` may differ from the
/// original input when wildcards, build metadata, or redundant unions are
/// normalized away.
///
/// # Examples
///
/// ```rust
/// use js_semver::Range;
///
/// assert_eq!(Range::parse("^1.2.3").unwrap().to_string(), ">=1.2.3 <2.0.0-0");
/// assert_eq!(Range::parse("^1.2.3 || *").unwrap().to_string(), "*");
/// assert_eq!(Range::parse("1.x.x+experimental").unwrap().to_string(), ">=1.0.0 <2.0.0-0");
/// ```
#[derive(Debug, Clone)]
pub struct Range {
    set: Vec<ComparatorSet>,
}

impl Range {
    /// Parse a range string.
    ///
    /// The parsed range is displayed in canonical comparator form.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::Range;
    ///
    /// assert_eq!(Range::parse("^1.2.3").unwrap().to_string(), ">=1.2.3 <2.0.0-0");
    /// assert_eq!(Range::parse(">=2.0.0").unwrap().to_string(), ">=2.0.0");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not a valid semver range string.
    pub fn parse(s: &str) -> Result<Self, SemverError> {
        parse_range(s)
    }

    /// Returns `true` if the given [`Version`] satisfies this range.
    ///
    /// This follows `node-semver`'s prerelease restriction rule: a prerelease
    /// version only matches when the range contains a comparator with the same
    /// `major.minor.patch` tuple and an explicit prerelease.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use js_semver::{Range, Version};
    ///
    /// let range = Range::parse("^1.2.3").unwrap();
    ///
    /// assert!(range.satisfies(&Version::parse("1.5.0").unwrap()));
    /// assert!(!range.satisfies(&Version::parse("2.0.0").unwrap()));
    /// ```
    #[must_use]
    pub fn satisfies(&self, version: &Version) -> bool {
        for comparator_set in &self.set {
            if comparator_set.test(version) {
                return true;
            }
        }
        false
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, cs) in self.set.iter().enumerate() {
            if i > 0 {
                f.write_str("||")?;
            }
            if cs.comparators.is_empty() {
                f.write_str("*")?;
            } else {
                for (j, c) in cs.comparators.iter().enumerate() {
                    if j > 0 {
                        f.write_str(" ")?;
                    }
                    write!(f, "{c}")?;
                }
            }
        }
        Ok(())
    }
}

impl FromStr for Range {
    type Err = SemverError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_range(s)
    }
}

// --------------------------------------------------------------------------
// Partial version (for range parsing)
// --------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Partial {
    major: Option<u64>,
    minor: Option<u64>,
    patch: Option<u64>,
    pre_release: PreRelease,
}

impl Partial {
    fn floor(self) -> Version {
        Version {
            major: self.major.unwrap_or(0),
            minor: self.minor.unwrap_or(0),
            patch: self.patch.unwrap_or(0),
            pre_release: self.pre_release,
            build: BuildMetadata::default(),
        }
    }
}

fn parse_partial(s: &str) -> Result<Partial, SemverError> {
    let s = s.trim().trim_start_matches(['v', '=']);
    let bytes = s.as_bytes();
    let mut core_end = bytes.len();
    let mut pre_start = None;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                core_end = i;
                break;
            }
            b'-' if pre_start.is_none() => pre_start = Some(i + 1),
            _ => {}
        }
        i += 1;
    }
    let version_end = pre_start.map_or(core_end, |start| start - 1);
    let version_core = &s[..version_end];
    let pre_part = pre_start.map(|start| &s[start..core_end]);

    if version_core.is_empty() && pre_part.is_some() {
        return Err(SemverError::new(format!("invalid version: {s}")));
    }
    if bytes.get(version_end.wrapping_sub(1)) == Some(&b'.') {
        return Err(SemverError::new(format!("trailing dot in: {s}")));
    }

    let (dot1, dot2) = find_component_dots(bytes, version_end, s)?;

    let major = parse_xr(match dot1 {
        Some(end) => &s[..end],
        None => version_core,
    })?;
    let minor = match (dot1, dot2) {
        (Some(start), Some(end)) => parse_xr(&s[start + 1..end])?,
        (Some(start), None) => parse_xr(&s[start + 1..version_end])?,
        (None, _) => None,
    };
    let patch = match dot2 {
        Some(start) => parse_xr(&s[start + 1..version_end])?,
        None => None,
    };

    let pre_release = if patch.is_some() {
        match pre_part {
            Some(p) if !p.is_empty() => PreRelease::new(p)?,
            Some(_) => return Err(SemverError::new(format!("empty pre-release in: {s}"))),
            None => PreRelease::default(),
        }
    } else {
        PreRelease::default()
    };

    Ok(Partial {
        major,
        minor,
        patch,
        pre_release,
    })
}

fn parse_xr(s: &str) -> Result<Option<u64>, SemverError> {
    match s {
        "" | "*" | "x" | "X" => Ok(None),
        _ => Ok(Some(parse_nr(s)?)),
    }
}

// --------------------------------------------------------------------------
// Comparator construction helpers (internal)
// --------------------------------------------------------------------------

fn release_version(major: u64, minor: u64, patch: u64) -> Version {
    Version::new(major, minor, patch)
}

fn find_component_dots(
    bytes: &[u8],
    version_end: usize,
    raw: &str,
) -> Result<(Option<usize>, Option<usize>), SemverError> {
    let mut first = None;
    let mut second = None;
    let mut pos = 0;
    while pos < version_end {
        if bytes[pos] == b'.' {
            if first.is_none() {
                first = Some(pos);
            } else if second.is_none() {
                second = Some(pos);
            } else {
                return Err(SemverError::new(format!(
                    "too many version components: {raw}"
                )));
            }
        }
        pos += 1;
    }
    Ok((first, second))
}

fn prerelease_version(major: u64, minor: u64, patch: u64, pre_release: PreRelease) -> Version {
    Version {
        major,
        minor,
        patch,
        pre_release,
        build: BuildMetadata::default(),
    }
}

const fn comparator_gte(ver: Version) -> Comparator {
    Comparator {
        op: Operator::GreaterThanOrEqual,
        version: ver,
    }
}
const fn comparator_gt(ver: Version) -> Comparator {
    Comparator {
        op: Operator::GreaterThan,
        version: ver,
    }
}
const fn comparator_lte(ver: Version) -> Comparator {
    Comparator {
        op: Operator::LessThanOrEqual,
        version: ver,
    }
}
const fn comparator_lt(ver: Version) -> Comparator {
    Comparator {
        op: Operator::LessThan,
        version: ver,
    }
}
const fn comparator_eq(ver: Version) -> Comparator {
    Comparator {
        op: Operator::Equal,
        version: ver,
    }
}

fn comparator_lt_upper_bound(major: u64, minor: u64, patch: u64) -> Comparator {
    comparator_lt(prerelease_version(major, minor, patch, PreRelease::zero()))
}

fn next_component(value: u64) -> Result<u64, SemverError> {
    if value >= crate::MAX_SAFE_INTEGER {
        return Err(SemverError::new(
            "range upper bound exceeds MAX_SAFE_INTEGER",
        ));
    }
    Ok(value + 1)
}

// --------------------------------------------------------------------------
// Range expansion helpers
// --------------------------------------------------------------------------

/// Expand a tilde range: `~1.2.3` → `>=1.2.3 <1.3.0-0`.
fn expand_tilde(p: Partial) -> Result<Vec<Comparator>, SemverError> {
    Ok(match (p.major, p.minor, p.patch) {
        (None, _, _) => vec![],
        (Some(0), None, _) => vec![comparator_lt_upper_bound(1, 0, 0)],
        (Some(maj), None, _) => vec![
            comparator_gte(release_version(maj, 0, 0)),
            comparator_lt_upper_bound(next_component(maj)?, 0, 0),
        ],
        (Some(maj), Some(mnr), None) => {
            vec![
                comparator_gte(release_version(maj, mnr, 0)),
                comparator_lt_upper_bound(maj, next_component(mnr)?, 0),
            ]
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            let floor = if p.pre_release.is_empty() {
                release_version(maj, mnr, patch)
            } else {
                prerelease_version(maj, mnr, patch, p.pre_release)
            };
            vec![
                comparator_gte(floor),
                comparator_lt_upper_bound(maj, next_component(mnr)?, 0),
            ]
        }
    })
}

/// Expand a caret range: `^1.2.3` → `>=1.2.3 <2.0.0-0`.
fn expand_caret(p: Partial) -> Result<Vec<Comparator>, SemverError> {
    Ok(match (p.major, p.minor, p.patch) {
        (None, _, _) => vec![],
        (Some(0), None, _) => vec![comparator_lt_upper_bound(1, 0, 0)],
        (Some(maj), None, _) => vec![
            comparator_gte(release_version(maj, 0, 0)),
            comparator_lt_upper_bound(next_component(maj)?, 0, 0),
        ],
        (Some(maj), Some(mnr), None) => {
            if maj > 0 {
                vec![
                    comparator_gte(release_version(maj, mnr, 0)),
                    comparator_lt_upper_bound(next_component(maj)?, 0, 0),
                ]
            } else if mnr > 0 {
                vec![
                    comparator_gte(release_version(0, mnr, 0)),
                    comparator_lt_upper_bound(0, next_component(mnr)?, 0),
                ]
            } else {
                vec![
                    comparator_gte(release_version(0, 0, 0)),
                    comparator_lt_upper_bound(0, 1, 0),
                ]
            }
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            let floor = if p.pre_release.is_empty() {
                release_version(maj, mnr, patch)
            } else {
                prerelease_version(maj, mnr, patch, p.pre_release)
            };
            if maj > 0 {
                vec![
                    comparator_gte(floor),
                    comparator_lt_upper_bound(next_component(maj)?, 0, 0),
                ]
            } else if mnr > 0 {
                vec![
                    comparator_gte(floor),
                    comparator_lt_upper_bound(0, next_component(mnr)?, 0),
                ]
            } else {
                vec![
                    comparator_gte(floor),
                    comparator_lt_upper_bound(0, 0, next_component(patch)?),
                ]
            }
        }
    })
}

/// Expand an x-range or primitive comparator to concrete [`Comparator`]s.
fn expand_primitive(op: Option<Operator>, p: Partial) -> Result<Vec<Comparator>, SemverError> {
    Ok(match op {
        // No operator or `=` → exact or x-range
        None | Some(Operator::Equal) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![],
            (Some(0), None, _) => vec![comparator_lt_upper_bound(1, 0, 0)],
            (Some(maj), None, _) => vec![
                comparator_gte(release_version(maj, 0, 0)),
                comparator_lt_upper_bound(next_component(maj)?, 0, 0),
            ],
            (Some(maj), Some(mnr), None) => {
                vec![
                    comparator_gte(release_version(maj, mnr, 0)),
                    comparator_lt_upper_bound(maj, next_component(mnr)?, 0),
                ]
            }
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    release_version(maj, mnr, patch)
                } else {
                    prerelease_version(maj, mnr, patch, p.pre_release)
                };
                vec![comparator_eq(ver)]
            }
        },
        Some(Operator::GreaterThan) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![comparator_lt_upper_bound(0, 0, 0)], // >* = impossible
            (Some(maj), None, _) => {
                vec![comparator_gte(release_version(next_component(maj)?, 0, 0))]
            }
            (Some(maj), Some(mnr), None) => {
                vec![comparator_gte(release_version(
                    maj,
                    next_component(mnr)?,
                    0,
                ))]
            }
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    release_version(maj, mnr, patch)
                } else {
                    prerelease_version(maj, mnr, patch, p.pre_release)
                };
                vec![comparator_gt(ver)]
            }
        },
        Some(Operator::GreaterThanOrEqual) => match (p.major, p.minor, p.patch) {
            (None, _, _) | (Some(0), None, _) => vec![],
            (Some(maj), None, _) => vec![comparator_gte(release_version(maj, 0, 0))],
            (Some(maj), Some(mnr), None) => {
                vec![comparator_gte(release_version(maj, mnr, 0))]
            }
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    release_version(maj, mnr, patch)
                } else {
                    prerelease_version(maj, mnr, patch, p.pre_release)
                };
                vec![comparator_gte(ver)]
            }
        },
        Some(Operator::LessThan) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![comparator_lt_upper_bound(0, 0, 0)], // <* = impossible
            (Some(maj), None, _) => vec![comparator_lt_upper_bound(maj, 0, 0)],
            (Some(maj), Some(mnr), None) => vec![comparator_lt_upper_bound(maj, mnr, 0)],
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    release_version(maj, mnr, patch)
                } else {
                    prerelease_version(maj, mnr, patch, p.pre_release)
                };
                vec![comparator_lt(ver)]
            }
        },
        Some(Operator::LessThanOrEqual) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![],
            (Some(maj), None, _) => {
                vec![comparator_lt_upper_bound(next_component(maj)?, 0, 0)]
            }
            (Some(maj), Some(mnr), None) => {
                vec![comparator_lt_upper_bound(maj, next_component(mnr)?, 0)]
            }
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    release_version(maj, mnr, patch)
                } else {
                    prerelease_version(maj, mnr, patch, p.pre_release)
                };
                vec![comparator_lte(ver)]
            }
        },
    })
}

/// Expand a hyphen range `a - b` to comparators.
fn expand_hyphen(a: Partial, b: Partial) -> Result<Vec<Comparator>, SemverError> {
    let lower = comparator_gte(a.floor());
    let upper = match (b.major, b.minor, b.patch) {
        (None, _, _) => None,
        (Some(maj), None, _) => Some(comparator_lt_upper_bound(next_component(maj)?, 0, 0)),
        (Some(maj), Some(mnr), None) => {
            Some(comparator_lt_upper_bound(maj, next_component(mnr)?, 0))
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            let ver = if b.pre_release.is_empty() {
                release_version(maj, mnr, patch)
            } else {
                prerelease_version(maj, mnr, patch, b.pre_release)
            };
            Some(comparator_lte(ver))
        }
    };
    let mut out = vec![lower];
    if let Some(upper) = upper {
        out.push(upper);
    }
    Ok(out)
}

// --------------------------------------------------------------------------
// Range parsing
// --------------------------------------------------------------------------

fn parse_range(s: &str) -> Result<Range, SemverError> {
    let s = s.trim();
    if s.len() > MAX_LENGTH {
        return Err(SemverError::new("range string too long"));
    }

    // Split by `||` inline, avoiding a Vec<&str> allocation.
    let bytes = s.as_bytes();
    let mut set = Vec::with_capacity(count_or_groups(bytes));
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'|' && bytes[i + 1] == b'|' {
            set.push(parse_comparator_set(s[start..i].trim())?);
            i += 2;
            start = i;
        } else {
            i += 1;
        }
    }
    set.push(parse_comparator_set(s[start..].trim())?);

    if set
        .iter()
        .any(|comparator_set| comparator_set.comparators.is_empty())
    {
        return Ok(Range {
            set: vec![ComparatorSet {
                comparators: vec![],
            }],
        });
    }

    set.dedup();

    Ok(Range { set })
}

fn parse_comparator_set(s: &str) -> Result<ComparatorSet, SemverError> {
    if s.is_empty() || s == "*" {
        return Ok(ComparatorSet {
            comparators: vec![],
        });
    }

    // Try hyphen range first.
    if let Some(comps) = try_hyphen(s)? {
        return Ok(ComparatorSet { comparators: comps });
    }

    let mut all = vec![];
    let bytes = s.as_bytes();
    let mut pos = 0;
    while let Some(t) = next_whitespace_token(s, bytes, &mut pos) {
        let is_op_only = matches!(t, ">" | ">=" | "<" | "<=" | "=" | "^" | "~" | "~=" | "~>");
        if is_op_only {
            if let Some(next) = next_whitespace_token(s, bytes, &mut pos) {
                // Concatenate op + version on the stack to avoid a heap allocation.
                // Operators are <=2 bytes; the whole range is <=MAX_LENGTH bytes.
                let mut buf = [0u8; 258];
                let op = t.as_bytes();
                let ver = next.as_bytes();
                let len = op.len() + ver.len();
                buf[..op.len()].copy_from_slice(op);
                buf[op.len()..len].copy_from_slice(ver);
                // SAFETY: `t` and `next` are slices of the original `&str`, so their bytes are
                // valid UTF-8 after concatenation as well.
                let merged = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                for comparator in parse_token(merged)? {
                    push_canonical_comparator(&mut all, comparator);
                }
            } else {
                // operator with no following token: just surface parse_token's error.
                drop(parse_token(t)?);
            }
        } else {
            for comparator in parse_token(t)? {
                push_canonical_comparator(&mut all, comparator);
            }
        }
    }
    Ok(ComparatorSet { comparators: all })
}

fn next_whitespace_token<'a>(s: &'a str, bytes: &[u8], pos: &mut usize) -> Option<&'a str> {
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
    if *pos >= bytes.len() {
        return None;
    }
    let start = *pos;
    while *pos < bytes.len() && !bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
    Some(&s[start..*pos])
}

fn count_or_groups(bytes: &[u8]) -> usize {
    let mut count = 1;
    let mut pos = 0;
    while pos + 1 < bytes.len() {
        if bytes[pos] == b'|' && bytes[pos + 1] == b'|' {
            count += 1;
            pos += 2;
        } else {
            pos += 1;
        }
    }
    count
}

fn compare_core_and_prerelease(left: &Version, right: &Version) -> core::cmp::Ordering {
    match left.major.cmp(&right.major) {
        core::cmp::Ordering::Equal => {}
        ordering @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => {
            return ordering;
        }
    }
    match left.minor.cmp(&right.minor) {
        core::cmp::Ordering::Equal => {}
        ordering @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => {
            return ordering;
        }
    }
    match left.patch.cmp(&right.patch) {
        core::cmp::Ordering::Equal => {}
        ordering @ (core::cmp::Ordering::Less | core::cmp::Ordering::Greater) => {
            return ordering;
        }
    }
    match (left.pre_release.is_empty(), right.pre_release.is_empty()) {
        (true, false) => core::cmp::Ordering::Greater,
        (false, true) => core::cmp::Ordering::Less,
        (true, true) => core::cmp::Ordering::Equal,
        (false, false) => left.pre_release.cmp_identifiers(&right.pre_release),
    }
}

/// Return `Some(comparators)` if `s` is a hyphen range `X - Y`, else `None`.
fn try_hyphen(s: &str) -> Result<Option<Vec<Comparator>>, SemverError> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b' ' && bytes[i + 1] == b'-' && bytes[i + 2] == b' ' {
            let left = s[..i].trim();
            let right = s[i + 3..].trim();
            let is_op = |c: char| matches!(c, '>' | '<' | '=' | '~' | '^');
            if !left.starts_with(is_op) && !right.starts_with(is_op) {
                let a = parse_partial(left)?;
                let b = parse_partial(right)?;
                return Ok(Some(expand_hyphen(a, b)?));
            }
        }
        i += 1;
    }
    Ok(None)
}

fn parse_token(s: &str) -> Result<Vec<Comparator>, SemverError> {
    let s = s.trim();
    if s.is_empty() || s == "*" {
        return Ok(vec![]);
    }

    // Standalone `-` is not a valid token (fragment of an incomplete hyphen range).
    if s == "-" {
        return Err(SemverError::new("invalid token: -"));
    }

    if let Some(rest) = s.strip_prefix('~') {
        let rest = rest.trim_start_matches(['=', '>']); // ~= and ~> are aliases for ~
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after ~"));
        }
        return expand_tilde(parse_partial(rest)?);
    }
    if let Some(rest) = s.strip_prefix('^') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after ^"));
        }
        return expand_caret(parse_partial(rest)?);
    }
    if let Some(rest) = s.strip_prefix(">=") {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after >="));
        }
        return expand_primitive(Some(Operator::GreaterThanOrEqual), parse_partial(rest)?);
    }
    if let Some(rest) = s.strip_prefix("<=") {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after <="));
        }
        return expand_primitive(Some(Operator::LessThanOrEqual), parse_partial(rest)?);
    }
    if let Some(rest) = s.strip_prefix('>') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after >"));
        }
        return expand_primitive(Some(Operator::GreaterThan), parse_partial(rest)?);
    }
    if let Some(rest) = s.strip_prefix('<') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after <"));
        }
        return expand_primitive(Some(Operator::LessThan), parse_partial(rest)?);
    }
    if let Some(rest) = s.strip_prefix('=') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after ="));
        }
        return expand_primitive(Some(Operator::Equal), parse_partial(rest)?);
    }

    expand_primitive(None, parse_partial(s)?)
}

fn push_canonical_comparator(all: &mut Vec<Comparator>, new: Comparator) {
    if is_impossible_comparator(&new) {
        all.clear();
        all.push(new);
        return;
    }
    for existing in &mut *all {
        if is_impossible_comparator(existing) {
            return;
        }
        match (existing.op, new.op) {
            (
                Operator::LessThan | Operator::LessThanOrEqual,
                Operator::LessThan | Operator::LessThanOrEqual,
            ) => {
                if existing.version.major == new.version.major
                    && existing.version.minor == new.version.minor
                    && existing.version.patch == new.version.patch
                {
                    let ordering = compare_core_and_prerelease(&existing.version, &new.version);
                    if ordering == core::cmp::Ordering::Greater
                        || (ordering == core::cmp::Ordering::Equal
                            && existing.op == Operator::LessThanOrEqual
                            && new.op == Operator::LessThan)
                    {
                        *existing = new;
                    }
                    return;
                }
            }
            (Operator::Equal, Operator::Equal)
                if compare_core_and_prerelease(&existing.version, &new.version)
                    == core::cmp::Ordering::Equal =>
            {
                return;
            }
            _ => {}
        }
    }
    all.push(new);
}

fn is_impossible_comparator(comparator: &Comparator) -> bool {
    comparator.op == Operator::LessThan
        && comparator.version.major == 0
        && comparator.version.minor == 0
        && comparator.version.patch == 0
        && comparator.version.pre_release == PreRelease::zero()
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    use core::fmt::{self, Write};

    use super::*;
    use crate::version::Version;

    fn v(s: &str) -> Version {
        s.parse().unwrap()
    }
    fn r(s: &str) -> Range {
        s.parse().unwrap()
    }

    fn assert_satisfies_case(range: &str, version: &str, expected: bool) {
        assert_eq!(r(range).satisfies(&v(version)), expected);
    }

    fn assert_display_case(input: &str, expected: &str) {
        assert_eq!(Range::parse(input).unwrap().to_string(), expected);
    }

    fn assert_invalid_range(input: &str) {
        assert!(Range::parse(input).is_err());
    }

    struct FailingWriter {
        fail_on: &'static str,
        fail_any: bool,
    }

    impl Write for FailingWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            if self.fail_any || s == self.fail_on {
                return Err(fmt::Error);
            }
            Ok(())
        }
    }

    // --- satisfies ---

    #[test]
    fn satisfies_cases() {
        assert_satisfies_case("^1.0.0", "1.2.3", true);
        assert_satisfies_case("^1.0.0", "1.9.9", true);
        assert_satisfies_case("^1.0.0", "2.0.0", false);
        assert_satisfies_case("^1.0.0", "0.9.9", false);
        assert_satisfies_case("~1.2.0", "1.2.3", true);
        assert_satisfies_case("~1.2.0", "1.2.9", true);
        assert_satisfies_case("~1.2.0", "1.3.0", false);
        assert_satisfies_case("~1.2.0", "1.1.9", false);
        assert_satisfies_case("1.0.0 - 2.0.0", "1.5.0", true);
        assert_satisfies_case("1.0.0 - 2.0.0", "1.0.0", true);
        assert_satisfies_case("1.0.0 - 2.0.0", "2.0.0", true);
        assert_satisfies_case("1.0.0 - 2.0.0", "3.0.0", false);
        assert_satisfies_case(">1.0.0", "2.0.0", true);
        assert_satisfies_case(">=1.0.0", "1.0.0", true);
        assert_satisfies_case("<1.0.0", "0.9.9", true);
        assert_satisfies_case("<=1.0.0", "1.0.0", true);
        assert_satisfies_case("=1.0.0", "1.0.0", true);
        assert_satisfies_case("1.0.0", "1.0.0", true);
        assert_satisfies_case("1.x", "1.2.3", true);
        assert_satisfies_case("1", "1.0.0", true);
        assert_satisfies_case("1", "1.9.9", true);
        assert_satisfies_case("1", "2.0.0", false);
        assert_satisfies_case("1.2.x", "1.2.9", true);
        assert_satisfies_case("*", "1.2.3", true);
        assert_satisfies_case("*", "0.0.1", true);
        assert_satisfies_case("1.0.0 || 2.0.0", "1.0.0", true);
        assert_satisfies_case("1.0.0 || 2.0.0", "2.0.0", true);
        assert_satisfies_case("1.0.0 || 2.0.0", "3.0.0", false);
    }

    #[test]
    fn prerelease_restriction() {
        assert!(!r("^1.0.0").satisfies(&v("1.0.0-alpha")));
        assert!(r(">=1.0.0-alpha").satisfies(&v("1.0.0-alpha.1")));
        assert!(r(">=1.0.0-alpha <=1.0.0-rc").satisfies(&v("1.0.0-beta")));
        assert!(!r(">=1.0.0-alpha <2.0.0").satisfies(&v("1.2.3-alpha")));
        // 4.0.0-rc.0 / rc.2: same [major,minor,patch] tuple → allowed
        assert!(r(">=4.0.0-rc.0").satisfies(&v("4.0.0-rc.0")));
        assert!(r(">=4.0.0-rc.0").satisfies(&v("4.0.0-rc.2")));
        // 4.2.0-rc.1: different tuple (4.2.0 ≠ 4.0.0) → excluded
        assert!(!r(">=4.0.0-rc.0").satisfies(&v("4.2.0-rc.1")));
    }

    #[test]
    fn parse_valid_and_display_cases() {
        assert_display_case("^1.0.0", ">=1.0.0 <2.0.0-0");
        assert_display_case("1.0.0", "1.0.0");
        assert_display_case("=1.0.0", "1.0.0");
        assert_display_case("~0.x.0", "<1.0.0-0");
        assert_display_case("~1.x.0", ">=1.0.0 <2.0.0-0");
        assert_display_case("*", "*");
        assert_display_case("* || ^1.2.3", "*");
        assert_display_case(">X", "<0.0.0-0");
        assert_display_case("<X", "<0.0.0-0");
        assert_display_case("<x <* || >* 2.x", "<0.0.0-0");
        assert_display_case(
            ">=1.0.0 <2.0.0 || >=2.0.0 <3.0.0",
            ">=1.0.0 <2.0.0||>=2.0.0 <3.0.0",
        );
        assert_display_case("~> 1", ">=1.0.0 <2.0.0-0");
        assert_display_case("~ 1.0", ">=1.0.0 <1.1.0-0");
        assert_display_case("~v0.5.2-pre", ">=0.5.2-pre <0.6.0-0");
        assert_display_case("^ 1.2.3", ">=1.2.3 <2.0.0-0");
        assert_display_case("<=1.2.3", "<=1.2.3");
        assert_display_case("<1.2.3", "<1.2.3");
        assert_display_case("x", "*");
        assert_display_case("=x", "*");

        assert!(Range::parse(">=1.0.0 <2.0.0").is_ok());
        assert!(try_hyphen(">=1.0.0 - 2.0.0").unwrap().is_none());
        assert!(try_hyphen("1.0.0 - <=2.0.0").unwrap().is_none());
        assert!(try_hyphen("1.2.3").unwrap().is_none());
        assert!(try_hyphen("1.2.3 -").unwrap().is_none());
        assert!(try_hyphen("- 1.2.3").unwrap().is_none());
    }

    // --- partial range syntax (tilde/caret with missing parts) ---

    #[test]
    fn tilde_partial() {
        // ~1 → >=1.0.0 <2.0.0-0
        assert!(r("~1").satisfies(&v("1.9.9")));
        assert!(!r("~1").satisfies(&v("2.0.0")));
        assert_eq!(Range::parse("~0.x.0").unwrap().to_string(), "<1.0.0-0");
        assert_eq!(
            Range::parse("~1.x.0").unwrap().to_string(),
            ">=1.0.0 <2.0.0-0"
        );
        // ~1.2 → >=1.2.0 <1.3.0-0
        assert!(r("~1.2").satisfies(&v("1.2.9")));
        assert!(!r("~1.2").satisfies(&v("1.3.0")));
        // ~1.2.3 with pre-release floor
        assert!(r("~1.2.3-alpha").satisfies(&v("1.2.3-beta")));
        assert!(!r("~1.2.3-alpha").satisfies(&v("1.3.0")));
    }

    #[test]
    fn caret_partial() {
        assert_eq!(Range::parse("^0").unwrap().to_string(), "<1.0.0-0");
        // ^1 → >=1.0.0 <2.0.0-0
        assert!(r("^1").satisfies(&v("1.9.9")));
        assert!(!r("^1").satisfies(&v("2.0.0")));
        // ^0.2 → >=0.2.0 <0.3.0
        assert!(r("^0.2").satisfies(&v("0.2.9")));
        assert!(!r("^0.2").satisfies(&v("0.3.0")));
        // ^0.0 → >=0.0.0 <0.1.0
        assert!(r("^0.0").satisfies(&v("0.0.9")));
        assert!(!r("^0.0").satisfies(&v("0.1.0")));
        // ^0.2.3 → >=0.2.3 <0.3.0
        assert!(r("^0.2.3").satisfies(&v("0.2.9")));
        assert!(!r("^0.2.3").satisfies(&v("0.3.0")));
        // ^0.0.3 → >=0.0.3 <0.0.4
        assert!(r("^0.0.3").satisfies(&v("0.0.3")));
        assert!(!r("^0.0.3").satisfies(&v("0.0.4")));
        // ^1.2.3-pre floor
        assert!(r("^1.2.3-alpha").satisfies(&v("1.2.3-beta")));
    }

    // --- primitive operators with partial versions ---

    #[test]
    fn primitive_partial() {
        // >1 → >=2.0.0
        assert!(r(">1").satisfies(&v("2.0.0")));
        assert!(!r(">1").satisfies(&v("1.9.9")));
        // >1.2 → >=1.3.0
        assert!(r(">1.2").satisfies(&v("1.3.0")));
        assert!(!r(">1.2").satisfies(&v("1.2.9")));
        // >=1.2 → >=1.2.0
        assert!(r(">=1.2").satisfies(&v("1.2.0")));
        // <1 → <1.0.0
        assert!(r("<1").satisfies(&v("0.9.9")));
        assert!(!r("<1").satisfies(&v("1.0.0")));
        // <1.2 → <1.2.0
        assert!(r("<1.2").satisfies(&v("1.1.9")));
        // <=1.2 → <1.3.0-0
        assert!(r("<=1.2").satisfies(&v("1.2.9")));
        assert!(!r("<=1.2").satisfies(&v("1.3.0")));
    }

    // --- Operator Display ---

    #[test]
    fn operator_display() {
        assert_eq!(Operator::LessThan.to_string(), "<");
        assert_eq!(Operator::LessThanOrEqual.to_string(), "<=");
        assert_eq!(Operator::GreaterThan.to_string(), ">");
        assert_eq!(Operator::GreaterThanOrEqual.to_string(), ">=");
        assert_eq!(Operator::Equal.to_string(), "=");
    }

    // --- wildcard operator forms ---

    #[test]
    fn wildcard_operator_forms() {
        // ~* → empty comparators → matches everything
        assert!(r("~*").satisfies(&v("1.0.0")));
        assert!(!r("~*").satisfies(&v("1.0.0-alpha")));
        // ^* → empty comparators → matches everything
        assert!(r("^*").satisfies(&v("1.0.0")));
        assert!(!r("^*").satisfies(&v("1.0.0-alpha")));
        // >=* → empty comparators → matches everything
        assert!(r(">=*").satisfies(&v("1.0.0")));
        assert!(!r(">=*").satisfies(&v("1.0.0-alpha")));
        // <=* → empty comparators → matches everything
        assert!(r("<=*").satisfies(&v("99.0.0")));
        assert!(!r("<=*").satisfies(&v("1.0.0-alpha")));
        assert!(!r("*").satisfies(&v("1.0.0-alpha")));
        // <* → c_lt(0.0.0) → impossible
        assert!(!r("<*").satisfies(&v("0.0.0")));
    }

    // --- caret with major.minor (no patch) ---

    #[test]
    fn caret_major_minor() {
        // ^1.2 with maj>0 → >=1.2.0 <2.0.0-0
        assert!(r("^1.2").satisfies(&v("1.9.9")));
        assert!(!r("^1.2").satisfies(&v("2.0.0")));
    }

    // --- primitive operators with single major ---

    #[test]
    fn primitive_single_major() {
        // >=1 → >=1.0.0
        assert!(r(">=1").satisfies(&v("1.0.0")));
        assert!(!r(">=1").satisfies(&v("0.9.9")));
        // <=1 → <2.0.0-0
        assert!(r("<=1").satisfies(&v("1.9.9")));
        assert!(!r("<=1").satisfies(&v("2.0.0")));
    }

    // --- exact match with pre-release ---

    #[test]
    fn eq_with_pre() {
        assert!(r("=1.2.3-alpha").satisfies(&v("1.2.3-alpha")));
        assert!(!r("=1.2.3-alpha").satisfies(&v("1.2.3-beta")));
    }

    // --- lt/gt with pre-release ---

    #[test]
    fn lt_gt_with_pre() {
        assert!(r("<1.2.3-beta").satisfies(&v("1.2.3-alpha")));
        assert!(!r("<1.2.3-beta").satisfies(&v("1.2.3-beta")));
        assert!(r(">1.2.3-alpha").satisfies(&v("1.2.3-beta")));
    }

    // --- hyphen range with partial upper bound ---

    #[test]
    fn hyphen_partial_upper() {
        // upper = * (None major) → no upper bound
        assert!(r("1.0.0 - *").satisfies(&v("99.0.0")));
        // upper = 2 (major only) → <3.0.0-0
        assert!(r("1.0.0 - 2").satisfies(&v("2.9.9")));
        assert!(!r("1.0.0 - 2").satisfies(&v("3.0.0")));
        // upper = 2.5 (major.minor) → <2.6.0
        assert!(r("1.0.0 - 2.5").satisfies(&v("2.5.9")));
        assert!(!r("1.0.0 - 2.5").satisfies(&v("2.6.0")));
        // upper with pre-release → <=2.0.0-alpha
        assert!(r("1.0.0 - 2.0.0-alpha").satisfies(&v("2.0.0-alpha")));
        assert!(!r("1.0.0 - 2.0.0-alpha").satisfies(&v("2.0.0")));
    }

    // --- range too long ---

    #[test]
    fn range_too_long() {
        assert!(Range::parse(&"^1.0.0 ".repeat(50)).is_err());
    }

    // --- parse_token with * mixed in ---

    #[test]
    fn parse_token_star_mixed() {
        // ">=1.0.0 *" → cs with only >=1.0.0 (star contributes nothing)
        assert!(r(">=1.0.0 *").satisfies(&v("1.0.0")));
        assert!(!r(">=1.0.0 *").satisfies(&v("0.9.9")));
    }

    #[test]
    fn parse_invalid_cases() {
        assert_invalid_range("01.0.0");
        assert_invalid_range("1a.0.0");
        assert_invalid_range("9007199254740992.0.0");
        assert_invalid_range(">");
        assert_invalid_range(">=");
        assert_invalid_range("> ");
        assert_invalid_range("<");
        assert_invalid_range("<=");
        assert_invalid_range("=");
        assert_invalid_range("^");
        assert_invalid_range("~");
        assert_invalid_range("~=");
        assert_invalid_range("1.0.0 -");
        assert_invalid_range("- 2.0.0");
        assert_invalid_range("1.0.0 - 2.0.0 - 3.0.0");
        assert_invalid_range(">>1.0.0");
        assert_invalid_range("><1.0.0");
        assert_invalid_range(">=<=1.0.0");
        assert_invalid_range("^01.0.0");
        assert_invalid_range("~01.0.0");
        assert_invalid_range(">01.0.0");
        assert_invalid_range(">=01.0.0");
        assert_invalid_range("^1.2.3.4");
        assert_invalid_range(">=a.b.c");
        assert_invalid_range(">1.2.3-0.01");
        assert_invalid_range("!!");
        assert_invalid_range("??");
        assert_invalid_range("1.0.0!");
        assert_invalid_range("1.0.0-");
        assert_invalid_range("-1.0.0");
        assert_invalid_range("^1.0.0-0.01");
        assert_invalid_range(">=1.0.0-01");
        assert_invalid_range("~1.0.0-01");
        assert_invalid_range("1.0.0>");
        assert_invalid_range("1.0.0>=");
        assert_invalid_range("1.0.0^");
        assert_invalid_range("~1.");
        assert_invalid_range("^1.");
        assert_invalid_range("^1.2.");
        assert_invalid_range("~1.2.");
        assert_invalid_range(">=1.");
        assert_invalid_range(">=1.2.");
        assert_invalid_range("1. - 2.0.0");
        assert_invalid_range("1.0.0 - 2.");
        assert_invalid_range("1.0.0- 2.0.0");
        assert_invalid_range("1.0.0 -2.0.0");
        assert_invalid_range("!1.0.0");
        assert_invalid_range("!=1.0.0");
        assert_invalid_range(">1.0.0\x00");
        assert_invalid_range("^00.0.0");
        assert_invalid_range("~0.00.0");
        assert_invalid_range(">=0.0.00");
        assert_invalid_range("^9007199254740991.0.0");
    }

    #[test]
    fn invalid_partial_after_operator_errors() {
        assert!(Range::parse("<=a.b.c").is_err());
        assert!(Range::parse("<a.b.c").is_err());
        assert!(Range::parse("=a.b.c").is_err());
        assert!(Range::parse("> a.b.c").is_err());
        assert!(Range::parse("^1.0.0 || >").is_err());
    }

    #[test]
    fn prerelease_zero_upper_bound_excludes_next_tuple_prereleases() {
        let range = r("^1.2.3");
        assert_eq!(range.to_string(), ">=1.2.3 <2.0.0-0");
        assert!(range.satisfies(&v("1.9.9")));
        assert!(!range.satisfies(&v("2.0.0-0")));
        assert!(!range.satisfies(&v("2.0.0-alpha")));
        assert!(!range.satisfies(&v("2.0.0")));
    }

    #[test]
    fn canonical_comparator_dedup_and_tightening() {
        assert_eq!(r("<1.2.4 <1.2.3").to_string(), "<1.2.4 <1.2.3");
        assert_eq!(r("<=1.2.3 <1.2.3").to_string(), "<1.2.3");
        assert_eq!(r("<1.2.3 <=1.2.3").to_string(), "<1.2.3");
        assert_eq!(r("1.2.3 1.2.3").to_string(), "1.2.3");
    }

    #[test]
    fn range_display_propagates_formatter_errors() {
        let mut wildcard_writer = FailingWriter {
            fail_on: "*",
            fail_any: false,
        };
        assert!(write!(&mut wildcard_writer, "{}", r("*")).is_err());

        let mut or_writer = FailingWriter {
            fail_on: "||",
            fail_any: false,
        };
        assert!(write!(&mut or_writer, "{}", r("1.0.0 || >=2.0.0")).is_err());

        let mut space_writer = FailingWriter {
            fail_on: " ",
            fail_any: false,
        };
        assert!(write!(&mut space_writer, "{}", r(">=1.0.0 <2.0.0")).is_err());

        let mut comparator_writer = FailingWriter {
            fail_on: "",
            fail_any: true,
        };
        assert!(write!(&mut comparator_writer, "{}", r(">=1.0.0")).is_err());
    }

    #[test]
    fn helper_branch_coverage_smoke() {
        assert_eq!(parse_partial("1.2").unwrap().minor, Some(2));

        assert_eq!(expand_tilde(parse_partial("1").unwrap()).unwrap().len(), 2);
        assert_eq!(
            expand_tilde(parse_partial("1.2").unwrap()).unwrap().len(),
            2
        );

        assert_eq!(expand_caret(parse_partial("1").unwrap()).unwrap().len(), 2);
        assert_eq!(
            expand_caret(parse_partial("1.2").unwrap()).unwrap().len(),
            2
        );
        assert_eq!(
            expand_caret(parse_partial("1.2.3").unwrap()).unwrap().len(),
            2
        );
        assert_eq!(
            expand_caret(parse_partial("0.2.3").unwrap()).unwrap().len(),
            2
        );

        assert_eq!(
            expand_primitive(None, parse_partial("1").unwrap())
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            expand_primitive(None, parse_partial("1.2").unwrap())
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            expand_primitive(Some(Operator::GreaterThan), parse_partial("1").unwrap())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            expand_primitive(Some(Operator::GreaterThan), parse_partial("1.2").unwrap())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            expand_primitive(
                Some(Operator::GreaterThanOrEqual),
                parse_partial("1").unwrap()
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            expand_primitive(
                Some(Operator::GreaterThanOrEqual),
                parse_partial("1.2").unwrap()
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            expand_primitive(Some(Operator::LessThanOrEqual), parse_partial("1").unwrap())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            expand_primitive(
                Some(Operator::LessThanOrEqual),
                parse_partial("1.2").unwrap()
            )
            .unwrap()
            .len(),
            1
        );

        assert_eq!(
            expand_hyphen(parse_partial("1.0.0").unwrap(), parse_partial("2").unwrap())
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            expand_hyphen(
                parse_partial("1.0.0").unwrap(),
                parse_partial("2.5").unwrap()
            )
            .unwrap()
            .len(),
            2
        );

        assert_eq!(
            parse_range("1.0.0||2.0.0").unwrap().to_string(),
            "1.0.0||2.0.0"
        );
        assert!(try_hyphen("1.0.0 - 2.0.0").unwrap().is_some());
    }
}
