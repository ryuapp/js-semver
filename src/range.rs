#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use core::fmt;
use core::str::FromStr;

use crate::error::SemverErrorKind;
use crate::identifier::{BuildMetadata, PreRelease};
use crate::version::{Version, compare_core_and_prerelease};
use crate::{MAX_LENGTH, SemverError};

mod expand;
mod partial;
#[cfg(test)]
mod tests;

#[cfg(test)]
use expand::{expand_caret, expand_primitive, expand_tilde};
use expand::{expand_caret_into, expand_hyphen, expand_primitive_into, expand_tilde_into};
#[cfg(test)]
use partial::{
    has_fully_qualified_numeric_core_after_full_strip, strip_build_metadata_and_find_prerelease,
};
use partial::{parse_partial, strip_build_metadata};

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
    set: ComparatorSets,
}

#[derive(Debug, Clone)]
enum ComparatorSets {
    One(ComparatorSet),
    Many(Vec<ComparatorSet>),
}

impl ComparatorSets {
    fn iter(&self) -> core::slice::Iter<'_, ComparatorSet> {
        match self {
            Self::One(set) => core::slice::from_ref(set).iter(),
            Self::Many(sets) => sets.iter(),
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::Many(sets) => sets.len(),
        }
    }
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
        for comparator_set in self.set.iter() {
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

// --------------------------------------------------------------------------
// Range parsing
// --------------------------------------------------------------------------

fn parse_range(s: &str) -> Result<Range, SemverError> {
    let s = s.trim();
    let exceeds_max_length =
        s.len() > MAX_LENGTH && range_len_without_build_metadata(s) > MAX_LENGTH && {
            let trimmed_prefix = s.trim_start_matches(['v', '=', '^', '~', '>', '<']);
            trimmed_prefix.len() > MAX_LENGTH
                && range_len_without_build_metadata(trimmed_prefix) > MAX_LENGTH
        };

    let bytes = s.as_bytes();
    let group_count = count_or_groups(bytes);
    if group_count == 1 {
        let comparator_set = parse_comparator_set(s, !exceeds_max_length)?;
        if !comparator_set.comparators.is_empty() && exceeds_max_length {
            return Err(SemverErrorKind::MaxLengthExceeded.into());
        }
        return Ok(Range {
            set: ComparatorSets::One(comparator_set),
        });
    }

    let mut set = Vec::with_capacity(group_count);
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'|' && bytes[i + 1] == b'|' {
            let comparator_set = parse_comparator_set(s[start..i].trim(), !exceeds_max_length)?;
            set.push(comparator_set);
            i += 2;
            start = i;
        } else {
            i += 1;
        }
    }
    set.push(parse_comparator_set(
        s[start..].trim(),
        !exceeds_max_length,
    )?);

    let has_unbounded_set = set
        .iter()
        .any(|comparator_set| comparator_set.comparators.is_empty());

    if has_unbounded_set && set.len() > 1 && exceeds_max_length {
        return Err(SemverErrorKind::MaxLengthExceeded.into());
    }

    if has_unbounded_set {
        return Ok(Range {
            set: ComparatorSets::One(ComparatorSet {
                comparators: vec![],
            }),
        });
    }

    if exceeds_max_length {
        return Err(SemverErrorKind::MaxLengthExceeded.into());
    }

    set.dedup();

    if set.len() == 1 {
        let comparator_set = set.remove(0);
        return Ok(Range {
            set: ComparatorSets::One(comparator_set),
        });
    }

    Ok(Range {
        set: ComparatorSets::Many(set),
    })
}

fn range_len_without_build_metadata(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut len = 0;
    let mut pos = 0;
    while pos < bytes.len() {
        if bytes[pos] == b'+' {
            pos += 1;
            while pos < bytes.len()
                && matches!(bytes[pos], b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'.')
            {
                pos += 1;
            }
        } else {
            len += 1;
            pos += 1;
        }
    }
    len
}

fn parse_comparator_set(s: &str, normalize: bool) -> Result<ComparatorSet, SemverError> {
    if s.is_empty() || s == "*" {
        return Ok(ComparatorSet {
            comparators: vec![],
        });
    }

    let bytes = s.as_bytes();
    if !bytes.iter().any(u8::is_ascii_whitespace) {
        let mut comparators = Vec::with_capacity(2);
        parse_token_into(&mut comparators, s)?;
        return Ok(ComparatorSet { comparators });
    }

    if let Some(comps) = try_hyphen(s)? {
        return Ok(ComparatorSet { comparators: comps });
    }

    let mut all = if normalize {
        Vec::with_capacity(count_whitespace_tokens(bytes).saturating_mul(2))
    } else {
        Vec::with_capacity(1)
    };
    let mut pos = 0;
    while let Some(t) = next_whitespace_token(s, bytes, &mut pos) {
        let is_op_only = matches!(t, ">" | ">=" | "<" | "<=" | "=" | "^" | "~" | "~=" | "~>");
        if is_op_only {
            if let Some(next) = next_whitespace_token(s, bytes, &mut pos) {
                let mut buf = [0u8; 258];
                let op = t.as_bytes();
                let ver = strip_build_metadata(next)?.as_bytes();
                let len = op.len() + ver.len();
                if len > buf.len() {
                    return Err(SemverErrorKind::MaxLengthExceeded.into());
                }
                buf[..op.len()].copy_from_slice(op);
                buf[op.len()..len].copy_from_slice(ver);
                // SAFETY: `t` and `next` are slices of the original `&str`, so their bytes are
                // valid UTF-8 after concatenation as well.
                let merged = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                parse_set_token_into(&mut all, merged, normalize)?;
            } else {
                parse_set_token_into(&mut all, t, normalize)?;
            }
        } else {
            parse_set_token_into(&mut all, t, normalize)?;
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

fn parse_set_token_into(
    all: &mut Vec<Comparator>,
    s: &str,
    normalize: bool,
) -> Result<(), SemverError> {
    if normalize {
        return parse_token_into(all, s);
    }

    let mut parsed = Vec::with_capacity(2);
    parse_token_into(&mut parsed, s)?;
    if all.is_empty() {
        all.extend(parsed.into_iter().take(1));
    }
    Ok(())
}

fn parse_token_into(all: &mut Vec<Comparator>, s: &str) -> Result<(), SemverError> {
    let s = s.trim();
    if s.is_empty() || s == "*" {
        return Ok(());
    }
    if s.starts_with('+') {
        strip_build_metadata(s)?;
        return Ok(());
    }

    if let Some(rest) = s.strip_prefix('~') {
        let rest = rest.trim_start_matches(['=', '>']); // ~= and ~> are aliases for ~
        return expand_tilde_into(all, parse_required_partial(rest, "~")?);
    }
    if let Some(rest) = s.strip_prefix('^') {
        return expand_caret_into(
            all,
            parse_required_partial(rest.trim_start_matches(['v', '=']), "^")?,
        );
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
