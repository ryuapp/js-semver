#[cfg(not(feature = "std"))]
use alloc::{format, vec, vec::Vec};

use core::fmt;
use core::str::FromStr;

use crate::version::{BuildMetadata, PreRelease, Version, parse_nr, parse_pre_release};
use crate::{MAX_LENGTH, SemverError};

// --------------------------------------------------------------------------
// Range types
// --------------------------------------------------------------------------

/// Comparison operator used in a version comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// `<` — less than
    Lt,
    /// `<=` — less than or equal to
    Lte,
    /// `>` — greater than
    Gt,
    /// `>=` — greater than or equal to
    Gte,
    /// `=` — exactly equal
    Eq,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Eq => "=",
        })
    }
}

#[derive(Debug, Clone)]
struct Comparator {
    op: Operator,
    version: Version,
}

impl Comparator {
    fn test(&self, v: &Version) -> bool {
        match self.op {
            Operator::Eq => v == &self.version,
            Operator::Gt => v > &self.version,
            Operator::Gte => v >= &self.version,
            Operator::Lt => v < &self.version,
            Operator::Lte => v <= &self.version,
        }
    }
}

impl fmt::Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op, self.version)
    }
}

#[derive(Debug, Clone)]
struct ComparatorSet {
    comparators: Vec<Comparator>,
}

impl ComparatorSet {
    fn test(&self, v: &Version) -> bool {
        if self.comparators.is_empty() {
            return true; // '*' matches everything
        }
        if !self.comparators.iter().all(|c| c.test(v)) {
            return false;
        }
        if !v.pre_release.is_empty() {
            return self.comparators.iter().any(|c| {
                c.version.major == v.major
                    && c.version.minor == v.minor
                    && c.version.patch == v.patch
                    && !c.version.pre_release.is_empty()
            });
        }
        true
    }
}

/// A version range, e.g. `^1.0.0` or `>=1.0.0 <2.0.0`.
#[derive(Debug, Clone)]
pub struct Range {
    set: Vec<ComparatorSet>,
}

impl Range {
    /// Parse a range string.
    ///
    /// # Errors
    ///
    /// Returns [`SemverError`] if `s` is not a valid semver range string.
    pub fn parse(s: &str) -> Result<Self, SemverError> {
        parse_range(s)
    }

    /// Returns `true` if `v` satisfies this range (any comparator set matches).
    #[must_use]
    pub fn satisfies(&self, v: &Version) -> bool {
        self.set.iter().any(|cs| cs.test(v))
    }

    /// Returns `true` if this range intersects with `other`.
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        range_intersects_impl(self, other)
    }

    /// Return the minimum version that satisfies this range, or `None`.
    #[must_use]
    pub fn min_version(&self) -> Option<Version> {
        let v000 = Version::new(0, 0, 0);
        if self.satisfies(&v000) {
            return Some(v000);
        }
        let v000_pre = vp(0, 0, 0, PreRelease::zero());
        if self.satisfies(&v000_pre) {
            return Some(v000_pre);
        }
        let mut candidates: Vec<Version> = vec![];
        for cs in &self.set {
            for c in &cs.comparators {
                if let Some(cand) = lower_bound_candidate(c) {
                    candidates.push(cand);
                }
            }
        }
        candidates.sort();
        candidates.dedup_by(|a, b| a == b);
        candidates.into_iter().find(|cand| self.satisfies(cand))
    }

    /// Return the highest version in `versions` that satisfies this range, or `None`.
    #[must_use]
    pub fn max_satisfying<'a>(&self, versions: &'a [Version]) -> Option<&'a Version> {
        versions.iter().filter(|v| self.satisfies(v)).max()
    }

    /// Return the lowest version in `versions` that satisfies this range, or `None`.
    #[must_use]
    pub fn min_satisfying<'a>(&self, versions: &'a [Version]) -> Option<&'a Version> {
        versions.iter().filter(|v| self.satisfies(v)).min()
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, cs) in self.set.iter().enumerate() {
            if i > 0 {
                f.write_str(" || ")?;
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
    let s = s.trim();
    let (s_no_build, _) = split_first(s, '+');
    let (version_core, pre_part) = split_first(s_no_build, '-');

    if version_core.is_empty() && pre_part.is_some() {
        return Err(SemverError::new(format!("invalid version: {s}")));
    }
    if !version_core.is_empty() && version_core.ends_with('.') {
        return Err(SemverError::new(format!("trailing dot in: {s}")));
    }
    // Parse up to 3 dot-separated components; reject a 4th to avoid Vec allocation.
    let mut dots = version_core.splitn(4, '.');
    let major = parse_xr(dots.next().unwrap_or(""))?;
    let minor = if let Some(t) = dots.next() {
        parse_xr(t)?
    } else {
        None
    };
    let patch = if let Some(t) = dots.next() {
        parse_xr(t)?
    } else {
        None
    };
    if dots.next().is_some() {
        return Err(SemverError::new(format!(
            "too many version components: {s}"
        )));
    }

    let pre_release = if patch.is_some() {
        match pre_part {
            Some(p) if !p.is_empty() => parse_pre_release(p)?,
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

fn split_first(s: &str, sep: char) -> (&str, Option<&str>) {
    s.find(sep)
        .map_or((s, None), |i| (&s[..i], Some(&s[i + sep.len_utf8()..])))
}

// --------------------------------------------------------------------------
// Comparator construction helpers (internal)
// --------------------------------------------------------------------------

fn v(major: u64, minor: u64, patch: u64) -> Version {
    Version::new(major, minor, patch)
}

fn vp(major: u64, minor: u64, patch: u64, pre_release: PreRelease) -> Version {
    Version {
        major,
        minor,
        patch,
        pre_release,
        build: BuildMetadata::default(),
    }
}

const fn c_gte(ver: Version) -> Comparator {
    Comparator {
        op: Operator::Gte,
        version: ver,
    }
}
const fn c_gt(ver: Version) -> Comparator {
    Comparator {
        op: Operator::Gt,
        version: ver,
    }
}
const fn c_lte(ver: Version) -> Comparator {
    Comparator {
        op: Operator::Lte,
        version: ver,
    }
}
const fn c_lt(ver: Version) -> Comparator {
    Comparator {
        op: Operator::Lt,
        version: ver,
    }
}
const fn c_eq(ver: Version) -> Comparator {
    Comparator {
        op: Operator::Eq,
        version: ver,
    }
}

// --------------------------------------------------------------------------
// Range expansion helpers
// --------------------------------------------------------------------------

/// Expand a tilde range: `~1.2.3` → `>=1.2.3 <1.3.0`
fn expand_tilde(p: Partial) -> Vec<Comparator> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => vec![],
        (Some(maj), None, _) => vec![c_gte(v(maj, 0, 0)), c_lt(v(maj + 1, 0, 0))],
        (Some(maj), Some(mnr), None) => vec![c_gte(v(maj, mnr, 0)), c_lt(v(maj, mnr + 1, 0))],
        (Some(maj), Some(mnr), Some(patch)) => {
            let floor = if p.pre_release.is_empty() {
                v(maj, mnr, patch)
            } else {
                vp(maj, mnr, patch, p.pre_release)
            };
            vec![c_gte(floor), c_lt(v(maj, mnr + 1, 0))]
        }
    }
}

/// Expand a caret range: `^1.2.3` → `>=1.2.3 <2.0.0`
fn expand_caret(p: Partial) -> Vec<Comparator> {
    match (p.major, p.minor, p.patch) {
        (None, _, _) => vec![],
        (Some(maj), None, _) => vec![c_gte(v(maj, 0, 0)), c_lt(v(maj + 1, 0, 0))],
        (Some(maj), Some(mnr), None) => {
            if maj > 0 {
                vec![c_gte(v(maj, mnr, 0)), c_lt(v(maj + 1, 0, 0))]
            } else if mnr > 0 {
                vec![c_gte(v(0, mnr, 0)), c_lt(v(0, mnr + 1, 0))]
            } else {
                vec![c_gte(v(0, 0, 0)), c_lt(v(0, 1, 0))]
            }
        }
        (Some(maj), Some(mnr), Some(patch)) => {
            let floor = if p.pre_release.is_empty() {
                v(maj, mnr, patch)
            } else {
                vp(maj, mnr, patch, p.pre_release)
            };
            if maj > 0 {
                vec![c_gte(floor), c_lt(v(maj + 1, 0, 0))]
            } else if mnr > 0 {
                vec![c_gte(floor), c_lt(v(0, mnr + 1, 0))]
            } else {
                vec![c_gte(floor), c_lt(v(0, 0, patch + 1))]
            }
        }
    }
}

/// Expand an x-range or primitive comparator to concrete [`Comparator`]s.
fn expand_primitive(op: Option<Operator>, p: Partial) -> Vec<Comparator> {
    match op {
        // No operator or `=` → exact or x-range
        None | Some(Operator::Eq) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![],
            (Some(maj), None, _) => vec![c_gte(v(maj, 0, 0)), c_lt(v(maj + 1, 0, 0))],
            (Some(maj), Some(mnr), None) => vec![c_gte(v(maj, mnr, 0)), c_lt(v(maj, mnr + 1, 0))],
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    v(maj, mnr, patch)
                } else {
                    vp(maj, mnr, patch, p.pre_release)
                };
                vec![c_eq(ver)]
            }
        },
        Some(Operator::Gt) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![c_lt(v(0, 0, 0))], // >* = impossible
            (Some(maj), None, _) => vec![c_gte(v(maj + 1, 0, 0))],
            (Some(maj), Some(mnr), None) => vec![c_gte(v(maj, mnr + 1, 0))],
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    v(maj, mnr, patch)
                } else {
                    vp(maj, mnr, patch, p.pre_release)
                };
                vec![c_gt(ver)]
            }
        },
        Some(Operator::Gte) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![],
            (Some(maj), None, _) => vec![c_gte(v(maj, 0, 0))],
            (Some(maj), Some(mnr), None) => vec![c_gte(v(maj, mnr, 0))],
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    v(maj, mnr, patch)
                } else {
                    vp(maj, mnr, patch, p.pre_release)
                };
                vec![c_gte(ver)]
            }
        },
        Some(Operator::Lt) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![c_lt(v(0, 0, 0))], // <* = impossible
            (Some(maj), None, _) => vec![c_lt(v(maj, 0, 0))],
            (Some(maj), Some(mnr), None) => vec![c_lt(v(maj, mnr, 0))],
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    v(maj, mnr, patch)
                } else {
                    vp(maj, mnr, patch, p.pre_release)
                };
                vec![c_lt(ver)]
            }
        },
        Some(Operator::Lte) => match (p.major, p.minor, p.patch) {
            (None, _, _) => vec![],
            (Some(maj), None, _) => vec![c_lt(v(maj + 1, 0, 0))],
            (Some(maj), Some(mnr), None) => vec![c_lt(v(maj, mnr + 1, 0))],
            (Some(maj), Some(mnr), Some(patch)) => {
                let ver = if p.pre_release.is_empty() {
                    v(maj, mnr, patch)
                } else {
                    vp(maj, mnr, patch, p.pre_release)
                };
                vec![c_lte(ver)]
            }
        },
    }
}

/// Expand a hyphen range `a - b` to comparators.
fn expand_hyphen(a: Partial, b: Partial) -> Vec<Comparator> {
    let lower = c_gte(a.floor());
    let upper = match (b.major, b.minor, b.patch) {
        (None, _, _) => None,
        (Some(maj), None, _) => Some(c_lt(v(maj + 1, 0, 0))),
        (Some(maj), Some(mnr), None) => Some(c_lt(v(maj, mnr + 1, 0))),
        (Some(maj), Some(mnr), Some(patch)) => {
            let ver = if b.pre_release.is_empty() {
                v(maj, mnr, patch)
            } else {
                vp(maj, mnr, patch, b.pre_release)
            };
            Some(c_lte(ver))
        }
    };
    let mut out = vec![lower];
    if let Some(u) = upper {
        out.push(u);
    }
    out
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
    let mut set = vec![];
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

    // Split by whitespace, then merge operator-only tokens with the following
    // version token so that e.g. `^ 1.0.0` is treated as `^1.0.0`.
    let mut iter = s.split_whitespace().peekable();
    let mut all = vec![];
    while let Some(t) = iter.next() {
        let is_op_only = matches!(t, ">" | ">=" | "<" | "<=" | "=" | "^" | "~" | "~=");
        if is_op_only {
            if let Some(&next) = iter.peek() {
                iter.next();
                // Concatenate op + version on the stack to avoid a heap allocation.
                // Operators are ≤2 bytes; the whole range is ≤MAX_LENGTH bytes.
                let mut buf = [0u8; 258];
                let op = t.as_bytes();
                let ver = next.as_bytes();
                let len = op.len() + ver.len();
                buf[..op.len()].copy_from_slice(op);
                buf[op.len()..len].copy_from_slice(ver);
                let merged = core::str::from_utf8(&buf[..len]).unwrap();
                all.extend(parse_token(merged)?);
            } else {
                // operator with no following token → let parse_token produce the error
                all.extend(parse_token(t)?);
            }
        } else {
            all.extend(parse_token(t)?);
        }
    }
    Ok(ComparatorSet { comparators: all })
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
                return Ok(Some(expand_hyphen(a, b)));
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
        let rest = rest.trim_start_matches('='); // ~= is alias for ~
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after ~"));
        }
        return Ok(expand_tilde(parse_partial(rest)?));
    }
    if let Some(rest) = s.strip_prefix('^') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after ^"));
        }
        return Ok(expand_caret(parse_partial(rest)?));
    }
    if let Some(rest) = s.strip_prefix(">=") {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after >="));
        }
        return Ok(expand_primitive(Some(Operator::Gte), parse_partial(rest)?));
    }
    if let Some(rest) = s.strip_prefix("<=") {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after <="));
        }
        return Ok(expand_primitive(Some(Operator::Lte), parse_partial(rest)?));
    }
    if let Some(rest) = s.strip_prefix('>') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after >"));
        }
        return Ok(expand_primitive(Some(Operator::Gt), parse_partial(rest)?));
    }
    if let Some(rest) = s.strip_prefix('<') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after <"));
        }
        return Ok(expand_primitive(Some(Operator::Lt), parse_partial(rest)?));
    }
    if let Some(rest) = s.strip_prefix('=') {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(SemverError::new("missing version after ="));
        }
        return Ok(expand_primitive(Some(Operator::Eq), parse_partial(rest)?));
    }

    Ok(expand_primitive(None, parse_partial(s)?))
}

// --------------------------------------------------------------------------
// Range intersection helpers
// --------------------------------------------------------------------------

fn range_intersects_impl(r1: &Range, r2: &Range) -> bool {
    r1.set
        .iter()
        .any(|cs1| r2.set.iter().any(|cs2| cs_intersect(cs1, cs2)))
}

fn cs_intersect(cs1: &ComparatorSet, cs2: &ComparatorSet) -> bool {
    if cs1.comparators.is_empty() || cs2.comparators.is_empty() {
        return true;
    }
    // Check lower-bound candidates from each set against the other.
    for c in &cs1.comparators {
        if let Some(cand) = lower_bound_candidate(c) {
            if cs1.test(&cand) && cs2.test(&cand) {
                return true;
            }
        }
    }
    for c in &cs2.comparators {
        if let Some(cand) = lower_bound_candidate(c) {
            if cs1.test(&cand) && cs2.test(&cand) {
                return true;
            }
        }
    }
    false
}

// --------------------------------------------------------------------------
// Private helpers for Range methods
// --------------------------------------------------------------------------

fn lower_bound_candidate(c: &Comparator) -> Option<Version> {
    match c.op {
        Operator::Eq | Operator::Gte => Some(c.version.clone()),
        Operator::Gt => {
            let mut ver = c.version.clone();
            ver.build = BuildMetadata::default();
            if ver.pre_release.is_empty() {
                ver.patch += 1;
            } else {
                ver.pre_release.push_numeric_zero();
            }
            Some(ver)
        }
        Operator::Lt | Operator::Lte => None,
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::{string::ToString, vec::Vec};

    use super::*;
    use crate::version::Version;

    fn v(s: &str) -> Version {
        s.parse().unwrap()
    }
    fn r(s: &str) -> Range {
        s.parse().unwrap()
    }

    // --- satisfies ---

    #[test]
    fn satisfies_caret() {
        assert!(r("^1.0.0").satisfies(&v("1.2.3")));
        assert!(r("^1.0.0").satisfies(&v("1.9.9")));
        assert!(!r("^1.0.0").satisfies(&v("2.0.0")));
        assert!(!r("^1.0.0").satisfies(&v("0.9.9")));
    }

    #[test]
    fn satisfies_tilde() {
        assert!(r("~1.2.0").satisfies(&v("1.2.3")));
        assert!(r("~1.2.0").satisfies(&v("1.2.9")));
        assert!(!r("~1.2.0").satisfies(&v("1.3.0")));
        assert!(!r("~1.2.0").satisfies(&v("1.1.9")));
    }

    #[test]
    fn satisfies_hyphen() {
        assert!(r("1.0.0 - 2.0.0").satisfies(&v("1.5.0")));
        assert!(r("1.0.0 - 2.0.0").satisfies(&v("1.0.0")));
        assert!(r("1.0.0 - 2.0.0").satisfies(&v("2.0.0")));
        assert!(!r("1.0.0 - 2.0.0").satisfies(&v("3.0.0")));
    }

    #[test]
    fn satisfies_primitive() {
        assert!(r(">1.0.0").satisfies(&v("2.0.0")));
        assert!(r(">=1.0.0").satisfies(&v("1.0.0")));
        assert!(r("<1.0.0").satisfies(&v("0.9.9")));
        assert!(r("<=1.0.0").satisfies(&v("1.0.0")));
        assert!(r("=1.0.0").satisfies(&v("1.0.0")));
        assert!(r("1.0.0").satisfies(&v("1.0.0")));
    }

    #[test]
    fn satisfies_xrange() {
        assert!(r("1.x").satisfies(&v("1.2.3")));
        assert!(r("1").satisfies(&v("1.0.0")));
        assert!(r("1").satisfies(&v("1.9.9")));
        assert!(!r("1").satisfies(&v("2.0.0")));
        assert!(r("1.2.x").satisfies(&v("1.2.9")));
        assert!(r("*").satisfies(&v("1.2.3")));
        assert!(r("*").satisfies(&v("0.0.1")));
    }

    #[test]
    fn satisfies_or() {
        assert!(r("1.0.0 || 2.0.0").satisfies(&v("1.0.0")));
        assert!(r("1.0.0 || 2.0.0").satisfies(&v("2.0.0")));
        assert!(!r("1.0.0 || 2.0.0").satisfies(&v("3.0.0")));
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

    // --- parse validity ---

    #[test]
    fn valid_ranges() {
        assert!("^1.0.0".parse::<Range>().is_ok());
        assert!("~1.2.3".parse::<Range>().is_ok());
        assert!("1.0.0 - 2.0.0".parse::<Range>().is_ok());
        assert!("*".parse::<Range>().is_ok());
    }

    // --- max/min satisfying ---

    #[test]
    fn max_satisfying_basic() {
        let vs: Vec<Version> = ["1.0.0", "1.2.0", "2.0.0", "3.0.0"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(r("^1.0.0").max_satisfying(&vs), Some(&v("1.2.0")));
        assert_eq!(r("^3.0.0").max_satisfying(&vs), Some(&v("3.0.0")));
        assert_eq!(r("^4.0.0").max_satisfying(&vs), None);
    }

    #[test]
    fn min_satisfying_basic() {
        let vs: Vec<Version> = ["1.0.0", "1.2.0", "2.0.0", "3.0.0"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(r("^1.0.0").min_satisfying(&vs), Some(&v("1.0.0")));
        assert_eq!(r(">=2.0.0").min_satisfying(&vs), Some(&v("2.0.0")));
    }

    // --- min_version ---

    #[test]
    fn min_version_basic() {
        assert_eq!(r(">=1.0.0").min_version(), Some(v("1.0.0")));
        assert_eq!(r("^1.2.3").min_version(), Some(v("1.2.3")));
        assert_eq!(r("~2.0.0").min_version(), Some(v("2.0.0")));
        assert_eq!(r("*").min_version(), Some(v("0.0.0")));
        assert_eq!(r("<2.0.0").min_version(), Some(v("0.0.0")));
    }

    // --- intersects ---

    #[test]
    fn intersects_ranges() {
        assert!(r("^1.0.0").intersects(&r("^1.5.0")));
        assert!(!r("^1.0.0").intersects(&r("^2.0.0")));
        assert!(r(">=1.0.0").intersects(&r("<=2.0.0")));
    }

    // --- Range::parse static method + Display ---

    #[test]
    fn parse_static_and_display() {
        // Display outputs canonical form (expanded comparators)
        let range = Range::parse("^1.0.0").unwrap();
        assert_eq!(range.to_string(), ">=1.0.0 <2.0.0");
        assert!(Range::parse(">=1.0.0 <2.0.0").is_ok());
    }

    // --- partial range syntax (tilde/caret with missing parts) ---

    #[test]
    fn tilde_partial() {
        // ~1 → >=1.0.0 <2.0.0
        assert!(r("~1").satisfies(&v("1.9.9")));
        assert!(!r("~1").satisfies(&v("2.0.0")));
        // ~1.2 → >=1.2.0 <1.3.0
        assert!(r("~1.2").satisfies(&v("1.2.9")));
        assert!(!r("~1.2").satisfies(&v("1.3.0")));
        // ~1.2.3 with pre-release floor
        assert!(r("~1.2.3-alpha").satisfies(&v("1.2.3-beta")));
        assert!(!r("~1.2.3-alpha").satisfies(&v("1.3.0")));
    }

    #[test]
    fn caret_partial() {
        // ^1 → >=1.0.0 <2.0.0
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
        // <=1.2 → <1.3.0
        assert!(r("<=1.2").satisfies(&v("1.2.9")));
        assert!(!r("<=1.2").satisfies(&v("1.3.0")));
    }

    // --- min_version with pre-release lower bound ---

    #[test]
    fn min_version_prerelease() {
        assert_eq!(
            r(">=1.0.0-alpha").min_version(),
            Some("1.0.0-alpha".parse().unwrap())
        );
        // >* is impossible, returns None
        assert_eq!(r(">*").min_version(), None);
    }

    // --- lower_bound_candidate with Gt + pre-release ---

    #[test]
    fn intersects_gt_pre() {
        assert!(r(">1.0.0-alpha").intersects(&r("^1.0.0")));
    }

    // --- Operator Display ---

    #[test]
    fn operator_display() {
        assert_eq!(Operator::Lt.to_string(), "<");
        assert_eq!(Operator::Lte.to_string(), "<=");
        assert_eq!(Operator::Gt.to_string(), ">");
        assert_eq!(Operator::Gte.to_string(), ">=");
        assert_eq!(Operator::Eq.to_string(), "=");
    }

    // --- min_version: 0.0.0-0 path ---

    #[test]
    fn min_version_v000_pre() {
        // 0.0.0 fails (pre-release restriction), but 0.0.0-0 passes
        assert_eq!(
            r(">=0.0.0-0 <=0.0.0-beta").min_version(),
            Some("0.0.0-0".parse().unwrap())
        );
    }

    // --- min_version: Gt with non-pre patch+1 ---

    #[test]
    fn min_version_gt() {
        assert_eq!(r(">1.0.0").min_version(), Some(v("1.0.1")));
    }

    // --- cs_intersect: wildcard range ---

    #[test]
    fn intersects_wildcard() {
        assert!(r("*").intersects(&r("^1.0.0")));
    }

    // --- wildcard operator forms ---

    #[test]
    fn wildcard_operator_forms() {
        // ~* → empty comparators → matches everything
        assert!(r("~*").satisfies(&v("1.0.0")));
        // ^* → empty comparators → matches everything
        assert!(r("^*").satisfies(&v("1.0.0")));
        // >=* → empty comparators → matches everything
        assert!(r(">=*").satisfies(&v("1.0.0")));
        // <=* → empty comparators → matches everything
        assert!(r("<=*").satisfies(&v("99.0.0")));
        // <* → c_lt(0.0.0) → impossible
        assert!(!r("<*").satisfies(&v("0.0.0")));
    }

    // --- caret with major.minor (no patch) ---

    #[test]
    fn caret_major_minor() {
        // ^1.2 with maj>0 → >=1.2.0 <2.0.0
        assert!(r("^1.2").satisfies(&v("1.9.9")));
        assert!(!r("^1.2").satisfies(&v("2.0.0")));
    }

    // --- primitive operators with single major ---

    #[test]
    fn primitive_single_major() {
        // >=1 → >=1.0.0
        assert!(r(">=1").satisfies(&v("1.0.0")));
        assert!(!r(">=1").satisfies(&v("0.9.9")));
        // <=1 → <2.0.0
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
        // upper = 2 (major only) → <3.0.0
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

    // --- range parse errors ---

    #[test]
    fn range_parse_errors() {
        // leading zero in partial version
        assert!(Range::parse("01.0.0").is_err());
        // non-numeric in version part
        assert!(Range::parse("1a.0.0").is_err());
        // major exceeds MAX_SAFE_INTEGER
        assert!(Range::parse("9007199254740992.0.0").is_err());
        // operator with no version
        assert!(Range::parse(">").is_err());
        assert!(Range::parse(">=").is_err());
        assert!(Range::parse("> ").is_err());
        assert!(Range::parse("<").is_err());
        assert!(Range::parse("<=").is_err());
        assert!(Range::parse("=").is_err());
        // caret / tilde with no version
        assert!(Range::parse("^").is_err());
        assert!(Range::parse("~").is_err());
        assert!(Range::parse("~=").is_err());
        // incomplete hyphen range
        assert!(Range::parse("1.0.0 -").is_err());
        assert!(Range::parse("- 2.0.0").is_err());
        assert!(Range::parse("1.0.0 - 2.0.0 - 3.0.0").is_err());
        // consecutive / mixed operators
        assert!(Range::parse(">>1.0.0").is_err());
        assert!(Range::parse("><1.0.0").is_err());
        assert!(Range::parse(">=<=1.0.0").is_err());
        // leading zero in operator range
        assert!(Range::parse("^01.0.0").is_err());
        assert!(Range::parse("~01.0.0").is_err());
        assert!(Range::parse(">01.0.0").is_err());
        assert!(Range::parse(">=01.0.0").is_err());
        // invalid version inside range
        assert!(Range::parse("^1.2.3.4").is_err());
        assert!(Range::parse(">=a.b.c").is_err());
        assert!(Range::parse(">1.2.3-0.01").is_err());
        // garbage
        assert!(Range::parse("!!").is_err());
        assert!(Range::parse("??").is_err());
        assert!(Range::parse("1.0.0!").is_err());
        // trailing / leading hyphen (not a hyphen range)
        assert!(Range::parse("1.0.0-").is_err());
        assert!(Range::parse("-1.0.0").is_err());
        // version after operator with invalid pre-release leading zero
        assert!(Range::parse("^1.0.0-0.01").is_err());
        assert!(Range::parse(">=1.0.0-01").is_err());
        assert!(Range::parse("~1.0.0-01").is_err());
        // too many version components inside range
        assert!(Range::parse("^1.2.3.4").is_err());
        // operator appended after version (not a valid comparator)
        assert!(Range::parse("1.0.0>").is_err());
        assert!(Range::parse("1.0.0>=").is_err());
        assert!(Range::parse("1.0.0^").is_err());
        // trailing dot in partial (missing component after dot)
        assert!(Range::parse("~1.").is_err());
        assert!(Range::parse("^1.").is_err());
        assert!(Range::parse("^1.2.").is_err());
        assert!(Range::parse("~1.2.").is_err());
        assert!(Range::parse(">=1.").is_err());
        assert!(Range::parse(">=1.2.").is_err());
        // trailing dot in hyphen range operands
        assert!(Range::parse("1. - 2.0.0").is_err());
        assert!(Range::parse("1.0.0 - 2.").is_err());
        // hyphen range with space between version and hyphen
        assert!(Range::parse("1.0.0- 2.0.0").is_err());
        assert!(Range::parse("1.0.0 -2.0.0").is_err());
        // bang / not-equal (not supported)
        assert!(Range::parse("!1.0.0").is_err());
        assert!(Range::parse("!=1.0.0").is_err());
        // control character in range
        assert!(Range::parse(">1.0.0\x00").is_err());
        // leading zero in various operators
        assert!(Range::parse("^00.0.0").is_err());
        assert!(Range::parse("~0.00.0").is_err());
        assert!(Range::parse(">=0.0.00").is_err());
    }

    #[test]
    fn range_display_and_dedup_paths() {
        assert_eq!(Range::parse("*").unwrap().to_string(), "*");
        assert_eq!(
            Range::parse("* || ^1.2.3").unwrap().to_string(),
            "* || >=1.2.3 <2.0.0"
        );
        assert_eq!(
            Range::parse(">=1.0.0 <2.0.0 || >=2.0.0 <3.0.0")
                .unwrap()
                .to_string(),
            ">=1.0.0 <2.0.0 || >=2.0.0 <3.0.0"
        );
    }

    #[test]
    fn operator_without_rhs_errors() {
        assert!(parse_token(">").is_err());
        assert!(parse_token(">=").is_err());
        assert!(parse_token("<").is_err());
        assert!(parse_token("<=").is_err());
        assert!(parse_token("=").is_err());
        assert!(parse_token("^").is_err());
        assert!(parse_token("~").is_err());
        assert!(parse_token("~=").is_err());
    }

    #[test]
    fn hyphen_range_not_used_with_operators() {
        assert!(try_hyphen(">=1.0.0 - 2.0.0").unwrap().is_none());
        assert!(try_hyphen("1.0.0 - <=2.0.0").unwrap().is_none());
        assert_eq!(
            Range::parse("^ 1.2.3").unwrap().to_string(),
            ">=1.2.3 <2.0.0"
        );
    }

    #[test]
    fn min_version_dedups_duplicate_candidates() {
        assert_eq!(r(">=1.2.3 >=1.2.3").min_version(), Some(v("1.2.3")));
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
    fn operator_and_hyphen_edge_paths() {
        assert_eq!(r("<=1.2.3").to_string(), "<=1.2.3");
        assert_eq!(r("<1.2.3").to_string(), "<1.2.3");
        assert_eq!(r("=1.2.3").to_string(), "=1.2.3");
        assert_eq!(r("x").to_string(), "*");
        assert_eq!(r("=x").to_string(), "*");
        assert!(try_hyphen("1.2.3").unwrap().is_none());
        assert!(try_hyphen("1.2.3 -").unwrap().is_none());
        assert!(try_hyphen("- 1.2.3").unwrap().is_none());
    }
}
