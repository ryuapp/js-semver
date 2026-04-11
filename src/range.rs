#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use core::fmt;
use core::str::FromStr;

use crate::error::SemverErrorKind;
use crate::identifier::{BuildMetadata, PreRelease};
use crate::number::{MAX_SAFE_INTEGER, parse_nr};
use crate::version::{Version, compare_core_and_prerelease};
use crate::{MAX_LENGTH, SemverError};

// --------------------------------------------------------------------------
// Range types
// --------------------------------------------------------------------------

/// Comparison operator used in a version comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operator {
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
/// Its string form preserves the parsed comparator structure, but may differ
/// from the original input when wildcards, build metadata, or unrestricted
/// unions are simplified away.
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
        return Err(SemverErrorKind::MissingVersionSegment.into());
    }
    if bytes.get(version_end.wrapping_sub(1)) == Some(&b'.') {
        return Err(SemverErrorKind::TrailingDot.into());
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

    let pre_release = match pre_part {
        Some(p) if p.is_empty() => return Err(SemverErrorKind::EmptySegment.into()),
        Some(p) => {
            if major.is_none() || minor.is_none() || patch.is_none() {
                return Err(SemverErrorKind::MissingVersionSegment.into());
            }
            PreRelease::new(p)?
        }
        None => PreRelease::default(),
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

fn find_component_dots(
    bytes: &[u8],
    version_end: usize,
    _raw: &str,
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
                return Err(SemverErrorKind::UnexpectedDot.into());
            }
        }
        pos += 1;
    }
    Ok((first, second))
}

fn version_with_pre_release(
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: PreRelease,
) -> Version {
    if pre_release.is_empty() {
        return Version::new(major, minor, patch);
    }

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
    comparator_lt(version_with_pre_release(
        major,
        minor,
        patch,
        PreRelease::zero(),
    ))
}

fn next_component(value: u64) -> Result<u64, SemverError> {
    if value >= MAX_SAFE_INTEGER {
        return Err(SemverErrorKind::MaxSafeIntegerExceeded.into());
    }
    Ok(value + 1)
}

// --------------------------------------------------------------------------
// Range expansion helpers
// --------------------------------------------------------------------------

/// Expand a tilde range: `~1.2.3` → `>=1.2.3 <1.3.0-0`.
fn expand_tilde_into(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => {}
        (Some(0), None, _) => push_canonical_comparator(out, comparator_lt_upper_bound(1, 0, 0)),
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, 0, 0, PreRelease::default())),
            );
            push_canonical_comparator(out, comparator_lt_upper_bound(next_component(maj)?, 0, 0));
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, mnr, 0, PreRelease::default())),
            );
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, next_component(mnr)?, 0));
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            let floor = version_with_pre_release(maj, mnr, patch, p.pre_release);
            push_canonical_comparator(out, comparator_gte(floor));
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, next_component(mnr)?, 0));
        }
    }
    Ok(())
}

#[cfg(test)]
fn expand_tilde(p: Partial) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_tilde_into(&mut out, p)?;
    Ok(out)
}

/// Expand a caret range: `^1.2.3` → `>=1.2.3 <2.0.0-0`.
fn expand_caret_into(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => {}
        (Some(0), None, _) => push_canonical_comparator(out, comparator_lt_upper_bound(1, 0, 0)),
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, 0, 0, PreRelease::default())),
            );
            push_canonical_comparator(out, comparator_lt_upper_bound(next_component(maj)?, 0, 0));
        }
        (Some(maj), Some(mnr), None) => {
            if maj > 0 {
                push_canonical_comparator(
                    out,
                    comparator_gte(version_with_pre_release(maj, mnr, 0, PreRelease::default())),
                );
                push_canonical_comparator(
                    out,
                    comparator_lt_upper_bound(next_component(maj)?, 0, 0),
                );
            } else if mnr > 0 {
                push_canonical_comparator(
                    out,
                    comparator_gte(version_with_pre_release(0, mnr, 0, PreRelease::default())),
                );
                push_canonical_comparator(
                    out,
                    comparator_lt_upper_bound(0, next_component(mnr)?, 0),
                );
            } else {
                push_canonical_comparator(
                    out,
                    comparator_gte(version_with_pre_release(0, 0, 0, PreRelease::default())),
                );
                push_canonical_comparator(out, comparator_lt_upper_bound(0, 1, 0));
            }
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            let floor = version_with_pre_release(maj, mnr, patch, p.pre_release);
            push_canonical_comparator(out, comparator_gte(floor));
            if maj > 0 {
                push_canonical_comparator(
                    out,
                    comparator_lt_upper_bound(next_component(maj)?, 0, 0),
                );
            } else if mnr > 0 {
                push_canonical_comparator(
                    out,
                    comparator_lt_upper_bound(0, next_component(mnr)?, 0),
                );
            } else {
                push_canonical_comparator(
                    out,
                    comparator_lt_upper_bound(0, 0, next_component(patch)?),
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn expand_caret(p: Partial) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_caret_into(&mut out, p)?;
    Ok(out)
}

/// Expand an x-range or primitive comparator to concrete [`Comparator`]s.
fn expand_primitive_into(
    out: &mut Vec<Comparator>,
    op: Option<Operator>,
    p: Partial,
) -> Result<(), SemverError> {
    match op {
        None | Some(Operator::Equal) => expand_equal_primitive(out, p)?,
        Some(Operator::GreaterThan) => expand_greater_than_primitive(out, p)?,
        Some(Operator::GreaterThanOrEqual) => expand_greater_than_or_equal_primitive(out, p),
        Some(Operator::LessThan) => expand_less_than_primitive(out, p),
        Some(Operator::LessThanOrEqual) => expand_less_than_or_equal_primitive(out, p)?,
    }
    Ok(())
}

fn version_from_partial(p: Partial, major: u64, minor: u64, patch: u64) -> Version {
    version_with_pre_release(major, minor, patch, p.pre_release)
}

fn expand_equal_primitive(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => {}
        (Some(0), None, _) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(1, 0, 0));
        }
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, 0, 0, PreRelease::default())),
            );
            push_canonical_comparator(out, comparator_lt_upper_bound(next_component(maj)?, 0, 0));
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, mnr, 0, PreRelease::default())),
            );
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, next_component(mnr)?, 0));
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(out, comparator_eq(version_from_partial(p, maj, mnr, patch)));
        }
    }
    Ok(())
}

fn expand_greater_than_primitive(out: &mut Vec<Comparator>, p: Partial) -> Result<(), SemverError> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => push_canonical_comparator(out, comparator_lt_upper_bound(0, 0, 0)),
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(
                    next_component(maj)?,
                    0,
                    0,
                    PreRelease::default(),
                )),
            );
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(
                    maj,
                    next_component(mnr)?,
                    0,
                    PreRelease::default(),
                )),
            );
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(out, comparator_gt(version_from_partial(p, maj, mnr, patch)));
        }
    }
    Ok(())
}

fn expand_greater_than_or_equal_primitive(out: &mut Vec<Comparator>, p: Partial) {
    match (p.major, p.minor, p.patch) {
        (None, _, _) | (Some(0), None, _) => {}
        (Some(maj), None, _) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, 0, 0, PreRelease::default())),
            );
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_with_pre_release(maj, mnr, 0, PreRelease::default())),
            );
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(
                out,
                comparator_gte(version_from_partial(p, maj, mnr, patch)),
            );
        }
    }
}

fn expand_less_than_primitive(out: &mut Vec<Comparator>, p: Partial) {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => push_canonical_comparator(out, comparator_lt_upper_bound(0, 0, 0)),
        (Some(maj), None, _) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, 0, 0));
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, mnr, 0));
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(out, comparator_lt(version_from_partial(p, maj, mnr, patch)));
        }
    }
}

fn expand_less_than_or_equal_primitive(
    out: &mut Vec<Comparator>,
    p: Partial,
) -> Result<(), SemverError> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => {}
        (Some(maj), None, _) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(next_component(maj)?, 0, 0));
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, next_component(mnr)?, 0));
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            push_canonical_comparator(
                out,
                comparator_lte(version_from_partial(p, maj, mnr, patch)),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
fn expand_primitive(op: Option<Operator>, p: Partial) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_primitive_into(&mut out, op, p)?;
    Ok(out)
}

/// Expand a hyphen range `a - b` to comparators.
fn expand_hyphen_into(
    out: &mut Vec<Comparator>,
    a: Partial,
    b: Partial,
) -> Result<(), SemverError> {
    push_canonical_comparator(out, comparator_gte(a.floor()));
    match (b.major, b.minor, b.patch) {
        (None, _, _) => {}
        (Some(maj), None, _) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(next_component(maj)?, 0, 0));
        }
        (Some(maj), Some(mnr), None) => {
            push_canonical_comparator(out, comparator_lt_upper_bound(maj, next_component(mnr)?, 0));
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            let ver = version_with_pre_release(maj, mnr, patch, b.pre_release);
            push_canonical_comparator(out, comparator_lte(ver));
        }
    }
    Ok(())
}

fn expand_hyphen(a: Partial, b: Partial) -> Result<Vec<Comparator>, SemverError> {
    let mut out = Vec::with_capacity(2);
    expand_hyphen_into(&mut out, a, b)?;
    Ok(out)
}

// --------------------------------------------------------------------------
// Range parsing
// --------------------------------------------------------------------------

fn parse_range(s: &str) -> Result<Range, SemverError> {
    let s = s.trim();
    if s.len() > MAX_LENGTH {
        return Err(SemverErrorKind::MaxLengthExceeded.into());
    }

    let bytes = s.as_bytes();
    let mut set = Vec::with_capacity(count_or_groups(bytes));
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'|' && bytes[i + 1] == b'|' {
            let comparator_set = parse_comparator_set(s[start..i].trim())?;
            set.push(comparator_set);
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

    if let Some(comps) = try_hyphen(s)? {
        return Ok(ComparatorSet { comparators: comps });
    }

    let bytes = s.as_bytes();
    let mut all = Vec::with_capacity(count_whitespace_tokens(bytes).saturating_mul(2));
    let mut pos = 0;
    while let Some(t) = next_whitespace_token(s, bytes, &mut pos) {
        let is_op_only = matches!(t, ">" | ">=" | "<" | "<=" | "=" | "^" | "~" | "~=" | "~>");
        if is_op_only {
            if let Some(next) = next_whitespace_token(s, bytes, &mut pos) {
                let mut buf = [0u8; 258];
                let op = t.as_bytes();
                let ver = next.as_bytes();
                let len = op.len() + ver.len();
                buf[..op.len()].copy_from_slice(op);
                buf[op.len()..len].copy_from_slice(ver);
                // SAFETY: `t` and `next` are slices of the original `&str`, so their bytes are
                // valid UTF-8 after concatenation as well.
                let merged = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                parse_token_into(&mut all, merged)?;
            } else {
                parse_token_into(&mut all, t)?;
            }
        } else {
            parse_token_into(&mut all, t)?;
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

fn count_whitespace_tokens(bytes: &[u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while pos < bytes.len() {
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        count += 1;
        while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
    }
    count
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

fn parse_token_into(all: &mut Vec<Comparator>, s: &str) -> Result<(), SemverError> {
    let s = s.trim();
    if s.is_empty() || s == "*" {
        return Ok(());
    }

    if let Some(rest) = s.strip_prefix('~') {
        let rest = rest.trim_start_matches(['=', '>']); // ~= and ~> are aliases for ~
        return expand_tilde_into(all, parse_required_partial(rest, "~")?);
    }
    if let Some(rest) = s.strip_prefix('^') {
        return expand_caret_into(all, parse_required_partial(rest, "^")?);
    }
    if let Some(rest) = s.strip_prefix(">=") {
        return expand_primitive_into(
            all,
            Some(Operator::GreaterThanOrEqual),
            parse_required_partial(rest, ">=")?,
        );
    }
    if let Some(rest) = s.strip_prefix("<=") {
        return expand_primitive_into(
            all,
            Some(Operator::LessThanOrEqual),
            parse_required_partial(rest, "<=")?,
        );
    }
    if let Some(rest) = s.strip_prefix('>') {
        return expand_primitive_into(
            all,
            Some(Operator::GreaterThan),
            parse_required_partial(rest, ">")?,
        );
    }
    if let Some(rest) = s.strip_prefix('<') {
        return expand_primitive_into(
            all,
            Some(Operator::LessThan),
            parse_required_partial(rest, "<")?,
        );
    }
    if let Some(rest) = s.strip_prefix('=') {
        return expand_primitive_into(
            all,
            Some(Operator::Equal),
            parse_required_partial(rest, "=")?,
        );
    }

    expand_primitive_into(all, None, parse_partial(s)?)
}

fn parse_required_partial(s: &str, operator: &'static str) -> Result<Partial, SemverError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(SemverErrorKind::MissingVersionAfterOperator(operator).into());
    }
    parse_partial(s)
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

    use super::*;

    #[test]
    fn try_hyphen_rejects_non_hyphen_forms() {
        assert!(try_hyphen(">=1.0.0 - 2.0.0").unwrap().is_none());
        assert!(try_hyphen("1.0.0 - <=2.0.0").unwrap().is_none());
        assert!(try_hyphen("1.2.3").unwrap().is_none());
        assert!(try_hyphen("1.2.3 -").unwrap().is_none());
        assert!(try_hyphen("- 1.2.3").unwrap().is_none());
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

    #[test]
    fn helper_count_and_expand_tilde_caret_coverage() {
        assert_eq!(parse_partial("1.2").unwrap().minor, Some(2));
        assert_eq!(count_whitespace_tokens(b""), 0);
        assert_eq!(count_whitespace_tokens(b">=1.0.0 <2.0.0"), 2);
        assert_eq!(count_whitespace_tokens(b"  >=1.0.0   <2.0.0  "), 2);

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
    }

    #[test]
    fn helper_expand_primitive_equal_coverage() {
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
            expand_primitive(None, parse_partial("0").unwrap())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            expand_primitive(None, parse_partial("1.2.3-alpha").unwrap())
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn helper_expand_primitive_greater_coverage() {
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
            expand_primitive(Some(Operator::GreaterThan), parse_partial("*").unwrap())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            expand_primitive(
                Some(Operator::GreaterThan),
                parse_partial("1.2.3-alpha").unwrap()
            )
            .unwrap()
            .len(),
            1
        );
    }

    #[test]
    fn helper_expand_primitive_greater_equal_coverage() {
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
            expand_primitive(
                Some(Operator::GreaterThanOrEqual),
                parse_partial("*").unwrap()
            )
            .unwrap()
            .len(),
            0
        );
        assert_eq!(
            expand_primitive(
                Some(Operator::GreaterThanOrEqual),
                parse_partial("1.2.3-alpha").unwrap()
            )
            .unwrap()
            .len(),
            1
        );
    }

    #[test]
    fn helper_expand_primitive_less_coverage() {
        assert_eq!(
            expand_primitive(Some(Operator::LessThan), parse_partial("1").unwrap())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            expand_primitive(
                Some(Operator::LessThan),
                parse_partial("1.2.3-alpha").unwrap()
            )
            .unwrap()
            .len(),
            1
        );
    }

    #[test]
    fn helper_expand_primitive_less_equal_coverage() {
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
            expand_primitive(
                Some(Operator::LessThanOrEqual),
                parse_partial("1.2.3-alpha").unwrap()
            )
            .unwrap()
            .len(),
            1
        );
    }

    #[test]
    fn helper_expand_hyphen_and_parse_range_coverage() {
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
        assert!(try_hyphen("1.0.0 - 2.0.0").unwrap().is_some());
    }

    #[test]
    fn helper_expand_error_paths() {
        let partial = parse_partial("1.2").unwrap();
        assert_eq!(partial.major, Some(1));
        assert_eq!(partial.minor, Some(2));
        assert_eq!(partial.patch, None);
        assert!(expand_tilde(parse_partial("9007199254740991").unwrap()).is_err());
        assert!(expand_tilde(parse_partial("1.9007199254740991").unwrap()).is_err());
        assert!(expand_tilde(parse_partial("1.9007199254740991.0").unwrap()).is_err());

        assert!(expand_caret(parse_partial("9007199254740991").unwrap()).is_err());
        assert!(expand_caret(parse_partial("9007199254740991.1").unwrap()).is_err());
        assert!(expand_caret(parse_partial("0.9007199254740991").unwrap()).is_err());
        assert!(expand_caret(parse_partial("0.9007199254740991.1").unwrap()).is_err());
        assert!(expand_caret(parse_partial("0.0.9007199254740991").unwrap()).is_err());
        assert_eq!(
            expand_caret(parse_partial("1.2").unwrap()).unwrap().len(),
            2
        );
        assert_eq!(
            expand_caret(parse_partial("0.2").unwrap()).unwrap().len(),
            2
        );
        let mut out = Vec::new();
        expand_caret_into(&mut out, parse_partial("0.2").unwrap()).unwrap();
        assert_eq!(out.len(), 2);

        assert!(expand_primitive(None, parse_partial("9007199254740991").unwrap()).is_err());
        assert!(expand_primitive(None, parse_partial("1.9007199254740991").unwrap()).is_err());
        assert!(
            expand_primitive(
                Some(Operator::GreaterThan),
                parse_partial("9007199254740991").unwrap()
            )
            .is_err()
        );
        assert!(
            expand_primitive(
                Some(Operator::GreaterThan),
                parse_partial("1.9007199254740991").unwrap()
            )
            .is_err()
        );
        assert!(
            expand_primitive(
                Some(Operator::LessThanOrEqual),
                parse_partial("9007199254740991").unwrap()
            )
            .is_err()
        );
        assert!(
            expand_primitive(
                Some(Operator::LessThanOrEqual),
                parse_partial("1.9007199254740991").unwrap()
            )
            .is_err()
        );

        assert!(
            expand_hyphen(
                parse_partial("1.0.0").unwrap(),
                parse_partial("9007199254740991").unwrap()
            )
            .is_err()
        );
        assert!(
            expand_hyphen(
                parse_partial("1.0.0").unwrap(),
                parse_partial("1.9007199254740991").unwrap()
            )
            .is_err()
        );
        assert!(parse_partial("1.bad").is_err());
        assert!(parse_partial("1.2-rc.0").is_err());
        assert!(parse_partial("2.x-rc.0").is_err());
        assert_eq!(parse_range("1.0.0 || 2.0.0").unwrap().set.len(), 2);
        assert_eq!(parse_range("1.0.0 || 2.0.0 || 3.0.0").unwrap().set.len(), 3);
        assert!(parse_range(">= || 1.0.0").is_err());
        assert!(parse_range("1.0.0 || >=").is_err());
        assert_eq!(try_hyphen("1.0.0 - 2.0.0").unwrap().unwrap().len(), 2);
        assert!(try_hyphen("1.0.0 - 9007199254740991").is_err());
    }

    #[test]
    fn public_and_comparator_helpers_are_used_in_crate_tests() {
        let version = Version::parse("1.2.3").unwrap();
        let prerelease = Version::parse("1.2.3-alpha.1").unwrap();

        let eq = Comparator {
            op: Operator::Equal,
            version: version.clone(),
        };
        let lt = Comparator {
            op: Operator::LessThan,
            version: Version::parse("2.0.0").unwrap(),
        };
        let set = ComparatorSet {
            comparators: vec![eq.clone(), lt.clone()],
        };
        let range = Range::parse("1.2.3").unwrap();

        assert!(eq.test(&version));
        assert!(
            Comparator {
                op: Operator::GreaterThan,
                version: Version::parse("1.2.2").unwrap(),
            }
            .test(&version)
        );
        assert!(
            Comparator {
                op: Operator::LessThanOrEqual,
                version: version.clone(),
            }
            .test(&version)
        );
        assert_eq!(eq.to_string(), "1.2.3");
        assert!(set.test(&version));
        assert!(!set.test(&prerelease));
        assert_eq!(
            Range::parse("^1.2.3").unwrap().to_string(),
            ">=1.2.3 <2.0.0-0"
        );
        assert!(range.satisfies(&version));
        assert!(!Range::parse("2.x || 3.x").unwrap().satisfies(&version));
        assert_eq!(
            compare_core_and_prerelease(&version, &Version::parse("1.2.4").unwrap()),
            core::cmp::Ordering::Less
        );
    }

    #[test]
    fn comparator_set_test_covers_release_and_prerelease_paths() {
        let release = Version::parse("1.2.3").unwrap();
        let prerelease = Version::parse("1.2.3-alpha.1").unwrap();
        let next_release = Version::parse("1.2.4").unwrap();
        let matching_pre = Version::parse("1.2.3-alpha.0").unwrap();

        let empty = ComparatorSet {
            comparators: Vec::new(),
        };
        assert!(empty.test(&release));
        assert!(!empty.test(&prerelease));

        let release_ok = ComparatorSet {
            comparators: vec![Comparator {
                op: Operator::Equal,
                version: release.clone(),
            }],
        };
        assert!(release_ok.test(&release));
        assert!(!release_ok.test(&next_release));

        let prerelease_without_match = ComparatorSet {
            comparators: vec![Comparator {
                op: Operator::GreaterThanOrEqual,
                version: release.clone(),
            }],
        };
        assert!(!prerelease_without_match.test(&prerelease));

        let prerelease_passes_but_tuple_does_not_match = ComparatorSet {
            comparators: vec![
                Comparator {
                    op: Operator::GreaterThan,
                    version: Version::parse("1.0.0").unwrap(),
                },
                Comparator {
                    op: Operator::LessThanOrEqual,
                    version: Version::parse("2.0.0").unwrap(),
                },
            ],
        };
        assert!(!prerelease_passes_but_tuple_does_not_match.test(&prerelease));

        let prerelease_with_match = ComparatorSet {
            comparators: vec![Comparator {
                op: Operator::GreaterThanOrEqual,
                version: matching_pre,
            }],
        };
        assert!(prerelease_with_match.test(&prerelease));
    }

    #[test]
    fn compare_core_and_prerelease_covers_all_major_paths() {
        assert_eq!(
            compare_core_and_prerelease(
                &Version::parse("2.0.0").unwrap(),
                &Version::parse("1.9.9").unwrap()
            ),
            core::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_core_and_prerelease(
                &Version::parse("1.3.0").unwrap(),
                &Version::parse("1.2.9").unwrap()
            ),
            core::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_core_and_prerelease(
                &Version::parse("1.2.4").unwrap(),
                &Version::parse("1.2.3").unwrap()
            ),
            core::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_core_and_prerelease(
                &Version::parse("1.2.3").unwrap(),
                &Version::parse("1.2.3-alpha.1").unwrap()
            ),
            core::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_core_and_prerelease(
                &Version::parse("1.2.3-alpha.1").unwrap(),
                &Version::parse("1.2.3-alpha.2").unwrap()
            ),
            core::cmp::Ordering::Less
        );
        assert_eq!(
            compare_core_and_prerelease(
                &Version::parse("1.2.3-alpha.1").unwrap(),
                &Version::parse("1.2.3-alpha.1").unwrap()
            ),
            core::cmp::Ordering::Equal
        );
    }
}
